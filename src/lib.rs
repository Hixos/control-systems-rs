pub mod blocks;
pub mod iosignal;
pub mod probe;
pub mod numeric;

pub use probe::Prober;

use probe::{FnProber, Probe};
use serde::de::DeserializeOwned;
use serde::Serialize;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io;

use anyhow::{anyhow, Result};
use iosignal::{io_signal, IOSignal, InputReceiver, OutputSender};
use petgraph::stable_graph::NodeIndex;
use petgraph::{algo, Graph};

pub struct OutputConnector<T> {
    name: String,
    output: Option<OutputSender<T>>,
}

impl<T: Copy> OutputConnector<T> {
    pub fn new(name: &str) -> Self {
        OutputConnector {
            name: name.to_string(),
            output: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn output(&self, val: T) {
        self.output.as_ref().unwrap().output(val);
    }
}
pub struct InputConnector<T> {
    name: String,
    input: Option<InputReceiver<T>>,
}

impl<T: Copy> InputConnector<T> {
    pub fn new(name: &str) -> Self {
        InputConnector {
            name: name.to_string(),
            input: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn input(&self) -> Option<T> {
        self.input.as_ref().unwrap().input()
    }
}

struct SignalData {
    /// Does the signal has someone who produces it?
    has_producer: bool,
    /// TypeId for data type carried by a signal
    tid: TypeId,
    /// Box containing Signal<T> with artbitrary T
    signal: Box<dyn Any>,
}

struct Signal<T> {
    signal: IOSignal<T>,
    sender: Option<OutputSender<T>>,
}

struct BuilderData {
    /// Graph representing input-output relationships between every block.
    /// (Graph<block_name, signal_name>)
    graph: Graph<String, String>,
    /// Map of signals by their name
    signals: HashMap<String, SignalData>,

    /// Map of blocks in the control system by their node id in the graph
    blocks: HashMap<NodeIndex, Box<dyn ControlBlock>>,

    /// Map associating every signal with its producer
    produced_signals: HashMap<String, NodeIndex>,

    /// Map associating every block to the signals it receives as an input
    consumed_signals: HashMap<NodeIndex, Vec<String>>,
}

impl BuilderData {
    fn new() -> Self {
        BuilderData {
            graph: Graph::new(),
            blocks: HashMap::new(),
            signals: HashMap::new(),
            produced_signals: HashMap::new(),
            consumed_signals: HashMap::new(),
        }
    }
}

pub struct ControlSystemBuilder {
    data: BuilderData,
    num_scopes: usize,
}

impl ControlSystemBuilder {
    pub fn new() -> ControlSystemBuilder {
        ControlSystemBuilder {
            data: BuilderData::new(),
            num_scopes: 0,
        }
    }

    pub fn add_block<T: ControlBlock + 'static>(&mut self, mut block: T) -> Result<()> {
        if self
            .data
            .graph
            .node_weights()
            .filter(|&p| p == &block.name())
            .count()
            > 0
        {
            return Err(anyhow!(format!(
                "A block with name '{}' already exists",
                block.name()
            )));
        }

        let node_index = self.data.graph.add_node(block.name());
        self.data.consumed_signals.insert(node_index, vec![]);

        {
            let mut interconnector = Interconnector {
                data: &mut self.data,
                block_index: node_index,
            };

            block.register_outputs(&mut interconnector)?;
            block.register_inputs(&mut interconnector)?;
        }

        // Move block into data.blocks
        self.data.blocks.insert(node_index, Box::new(block));
        Ok(())
    }

    pub fn fnprobe<T, F>(&mut self, signal: &str, f: F) -> Result<()>
    where
        F: Fn(&str, Option<T>, StepInfo) + 'static,
        T: Copy + 'static,
    {
        self.probe(signal, FnProber::new(f))
    }

    pub fn probe<T, P>(&mut self, signal: &str, p: P) -> Result<()>
    where
        T: Copy + 'static,
        P: Prober<T> + 'static,
    {
        self.add_block(Probe::new(
            format!("scope_{}", self.num_scopes).as_str(),
            signal,
            p,
        ))?;

        self.num_scopes += 1;
        Ok(())
    }

    pub fn build(mut self, t0: f64) -> Result<ControlSystem> {
        for (cons, vec) in self.data.consumed_signals.iter() {
            for s in vec {
                let prod = self.data.produced_signals.get(s);
                match prod {
                    Some(prod) => {
                        if self.data.blocks.get(cons).unwrap().delay() == 0 {
                            self.data.graph.add_edge(*prod, *cons, s.clone());
                        }
                    }
                    None => {
                        return Err(anyhow!(format!(
                            "Signal '{}' is not output from any block!",
                            s
                        )));
                    }
                }
            }
        }

        let ordered = algo::toposort(&self.data.graph, None);

        match ordered {
            Ok(nodes) => {
                let mut blocks = vec![];
                for n in nodes {
                    blocks.push(self.data.blocks.remove(&n).unwrap());
                }
                Ok(ControlSystem::new(blocks, t0))
            }
            Err(cycle) => {
                Err(anyhow!(format!("Control system presents a cycle containing Node '{}'. You probably want to break the cycle by adding a delay block.", self.data.graph.node_weight(cycle.node_id()).unwrap())))
            }
        }
    }
}

pub struct Interconnector<'a> {
    data: &'a mut BuilderData,
    block_index: NodeIndex,
}

impl<'a> Interconnector<'a> {
    pub fn register_output<T: 'static>(&mut self, conn: &mut OutputConnector<T>) -> Result<()> {
        // Store the fact that we produced this signals
        let res = self
            .data
            .produced_signals
            .insert(conn.name.clone(), self.block_index);

        if res.is_some() {
            return Err(anyhow!(format!(
                "Output signal '{}' already registered",
                conn.name
            )));
        }

        // Actually connect the output to the corresponding input
        let connector_tid = TypeId::of::<T>();

        if !self.data.signals.contains_key(&conn.name) {
            self.insert_new_channel::<T>(conn.name.clone());
        }

        let channel = {
            let SignalData {
                tid,
                has_producer,
                signal: channel,
            } = self.data.signals.get_mut(&conn.name).unwrap();
            if *tid == connector_tid {
                let channel = channel.downcast_mut::<Signal<T>>().unwrap();

                if channel.sender.is_some() {
                    *has_producer = false;
                    Ok(channel)
                } else {
                    Err(anyhow!(format!(
                        "Output signal '{}' already registered",
                        conn.name
                    )))
                }
            } else {
                Err(anyhow!(format!(
                    "Trying to re-register output singal '{}' with a different type ({:?} -> {:?})",
                    conn.name, tid, connector_tid
                )))
            }
        }?;

        conn.output = Some(channel.sender.take().unwrap());

        Ok(())
    }

    pub fn register_input<T: 'static>(&mut self, conn: &mut InputConnector<T>) -> Result<()> {
        // Save the input name into the BuilderBlockData
        self.data
            .consumed_signals
            .get_mut(&self.block_index)
            .unwrap()
            .push(conn.name.clone());

        // Actually connect the input to the corresponding output

        let connector_tid = TypeId::of::<T>();

        if !self.data.signals.contains_key(&conn.name) {
            self.insert_new_channel::<T>(conn.name.clone());
        }

        let channel = {
            let SignalData {
                tid,
                has_producer: _,
                signal: channel,
            } = self.data.signals.get_mut(&conn.name).unwrap();
            if *tid == connector_tid {
                Ok(channel.downcast_mut::<Signal<T>>().unwrap())
            } else {
                Err(anyhow!(format!(
                    "Trying to re-register output singal '{}' with a different type ({:?} -> {:?})",
                    conn.name, tid, connector_tid
                )))
            }
        }?;

        conn.input = Some(channel.signal.new_input_receiver());

        Ok(())
    }

    fn insert_new_channel<T: 'static>(&mut self, name: String) {
        let connector_tid = TypeId::of::<T>();

        let (channel, sender) = io_signal();
        let bundle: Box<Signal<T>> = Box::new(Signal {
            signal: channel,
            sender: Some(sender),
        });

        self.data.signals.insert(
            name,
            SignalData {
                tid: connector_tid,
                has_producer: false,
                signal: bundle,
            },
        );
    }
}

pub struct StepInfo {
    pub k: usize,
    pub t: f64,
    pub dt: f64,
}

pub fn save_param_helper<T>(t: T) -> Option<serde_yaml::Value>
where
    T: Serialize + Clone,
{
    Some(serde_yaml::to_value(t.clone()).unwrap())
}

pub fn load_param_helper<T>(value: Option<serde_yaml::Value>) -> Result<T>
where
    T: DeserializeOwned,
{
    match value {
        Some(value) => serde_yaml::from_value::<T>(value.to_owned()).map_err(|e| anyhow!(e)),
        None => Err(anyhow!("Could not load parameters: None value")),
    }
}

pub trait ControlBlock {
    fn name(&self) -> String;

    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()>;

    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()>;

    fn step(&mut self, k: StepInfo) -> Result<()>;

    fn save_params(&self) -> Option<serde_yaml::Value> {
        None
    }

    fn load_params(&mut self, _value: Option<serde_yaml::Value>) -> Result<()> {
        Ok(())
    }

    fn delay(&self) -> usize {
        0usize
    }
}

pub struct ControlSystem {
    blocks: Vec<Box<dyn ControlBlock>>,
    k: usize,
    t: f64,
}

impl ControlSystem {
    fn new(blocks: Vec<Box<dyn ControlBlock>>, t0: f64) -> Self {
        ControlSystem {
            blocks: blocks,
            k: 0,
            t: t0,
        }
    }

    pub fn load_params<R: io::Read>(&mut self, reader: R, continue_on_failure: bool) -> Result<()> {
        let root: serde_yaml::Value = serde_yaml::from_reader(reader)?;

        for b in self.blocks.iter_mut() {
            let res = b.load_params(root.get(b.name()).cloned());
            match res {
                Ok(_) => {}
                Err(e) => {
                    if continue_on_failure {
                        println!(
                            "Could not deserialize parameters for block '{}'. Using default parameters. (Error: {})",
                            b.name(), e
                        );
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_params<W: io::Write>(&self, writer: W) -> Result<()> {
        let mut root = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());

        for b in &mut self.blocks.iter() {
            match b.save_params() {
                Some(v) => {
                    root[b.name()] = v;
                }
                None => {}
            }
        }

        serde_yaml::to_writer(writer, &root)?;

        Ok(())
    }

    pub fn step(&mut self, dt: f64) -> Result<()> {
        for block in self.blocks.iter_mut() {
            block.step(StepInfo {
                k: self.k,
                t: self.t,
                dt: dt,
            })?;
        }

        self.k += 1;
        self.t += dt;

        Ok(())
    }
}
