use std::collections::{HashMap, HashSet};

use petgraph::{algo::toposort, dot::Dot, prelude::NodeIndex, Graph};
use serde::{Deserialize, Serialize};

use crate::{
    controlblock::{Block, StepInfo, StepResult},
    io::AnySignal,
    ControlSystemError, ParameterStore, Result,
};

pub struct ControlSystem {
    name: String,
    #[allow(unused)]
    signals: HashMap<String, AnySignal>,
    blocks: Vec<Box<dyn Block>>,
    #[allow(unused)]
    graph: Graph<String, String>,

    #[allow(unused)]
    params: ControlSystemParameters,

    step: StepInfo,
}
#[derive(Serialize, Deserialize)]
pub struct ControlSystemParameters {
    pub dt: f64,
    /// Maximum number of iterations. 0 for unlimited
    pub max_iter: usize, 
}

impl ControlSystem {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn step(&mut self) -> Result<StepResult> {
        let mut stop = false;
        for b in self.blocks.iter_mut() {
            // In case of stop, complete this step and return it
            stop = stop || b.step(self.step)? == StepResult::Stop;
        }

        self.step.k += 1;
        self.step.t += self.step.dt;

        if stop || (self.params.max_iter > 0 && self.step.k > self.params.max_iter) {
            Ok(StepResult::Stop)
        } else {
            Ok(StepResult::Continue)
        }
    }
}

struct BlockData {
    block: Box<dyn Block>,
    registered_inputs: HashMap<String, String>,
    registered_outputs: HashMap<String, String>,
}

#[derive(Default)]
pub struct ControlSystemBuilder {
    signals: HashMap<String, AnySignal>,
    blocks: HashMap<String, BlockData>,
}

impl ControlSystemBuilder {
    pub fn add_block<T: Block + 'static>(
        &mut self,
        block: T,
        input_connections: &[(&str, &str)],
        output_connections: &[(&str, &str)],
    ) -> Result<&mut Self, ControlSystemError> {
        let name = block.name();

        if self.blocks.contains_key(&name) {
            return Err(ControlSystemError::DuplicateBlockName(name));
        }

        let mut block_data = BlockData {
            block: Box::new(block),
            registered_inputs: HashMap::new(),
            registered_outputs: HashMap::new(),
        };

        self.connect_inputs(&mut block_data, input_connections)?;
        self.connect_outputs(&mut block_data, output_connections)?;

        self.blocks.insert(block_data.block.name(), block_data);

        Ok(self)
    }

    pub fn build_from_store(
        self,
        name: &str,
        param_store: &mut ParameterStore,
        default_params: ControlSystemParameters,
    ) -> Result<ControlSystem, ControlSystemError> {
        let params = param_store.get_cs_params(default_params)?;
        self.build(name, params)
    }

    pub fn build(
        mut self,
        name: &str,
        params: ControlSystemParameters,
    ) -> Result<ControlSystem, ControlSystemError> {
        for (name, data) in self.blocks.iter_mut() {
            let mut input_signals = data.block.input_signals();

            for (signal, input) in data.registered_inputs.iter() {
                let signal = self
                    .signals
                    .get(signal)
                    .ok_or(ControlSystemError::UnknownSignal {
                        port: input.clone(),
                        signal: signal.clone(),
                        blockname: name.clone(),
                    })?;

                **input_signals.get_mut(input).unwrap() = Some(signal.clone());
            }
        }

        let graph_cyclic = self.build_graph(true);
        println!("{}", Dot::new(&graph_cyclic));

        let graph = self.build_graph(false);
        let sorted = toposort(&graph, None);

        match sorted {
            Ok(nodes) => {
                let mut blocks = vec![];
                for node_ix in nodes {
                    let node = graph.node_weight(node_ix).unwrap();
                    blocks.push(self.blocks.remove(node).unwrap().block);
                }

                let dt = params.dt;

                Ok(ControlSystem {
                    name: name.to_string(),
                    signals: self.signals,
                    blocks,
                    graph,
                    params,
                    step: StepInfo::new(dt),
                })
            }
            Err(cycle) => Err(ControlSystemError::CycleDetected(
                graph.node_weight(cycle.node_id()).unwrap().clone(),
            )),
        }
    }
}

impl ControlSystemBuilder {
    fn connect_outputs(
        &mut self,
        block_data: &mut BlockData,
        output_connections: &[(&str, &str)],
    ) -> Result<(), ControlSystemError> {
        let block_name = block_data.block.name();
        let mut output_signals = block_data.block.output_signals();

        for (port, signal_name) in output_connections.iter() {
            if self.signals.contains_key(*signal_name) {
                // A signal with the same name is already produced by another output
                return Err(ControlSystemError::MultipleProducers {
                    port: port.to_string(),
                    signal: signal_name.to_string(),
                    blockname: block_name.clone(),
                });
            } else {
                let signal =
                    output_signals
                        .get_mut(*port)
                        .ok_or(ControlSystemError::UnknownPort {
                            port: port.to_string(),
                            blockname: block_name.clone(),
                        })?;

                signal.set_name(signal_name);

                self.signals
                    .insert(signal_name.to_string(), (*(signal)).clone());
                block_data
                    .registered_outputs
                    .insert(port.to_string(), signal_name.to_string());
                output_signals.remove(*port);
            }
        }

        if output_signals.is_empty() {
            Ok(())
        } else {
            Err(ControlSystemError::UnconnectedPorts {
                ports: output_signals.keys().cloned().collect(),
                blockname: block_name.clone(),
            })
        }
    }

    fn connect_inputs(
        &mut self,
        block_data: &mut BlockData,
        input_connections: &[(&str, &str)],
    ) -> Result<(), ControlSystemError> {
        let mut input_signals: HashSet<String> =
            block_data.block.input_signals().into_keys().collect();

        for (port, signal) in input_connections {
            if input_signals.contains(*port) {
                block_data
                    .registered_inputs
                    .insert(signal.to_string(), port.to_string());
                input_signals.remove(*port);
            } else {
                return Err(ControlSystemError::UnknownPort {
                    port: port.to_string(),
                    blockname: block_data.block.name(),
                });
            }
        }

        if input_signals.is_empty() {
            Ok(())
        } else {
            Err(ControlSystemError::UnconnectedPorts {
                ports: input_signals.iter().cloned().collect(),
                blockname: block_data.block.name(),
            })
        }
    }

    fn build_graph(&self, cyclic_edges: bool) -> Graph<String, String> {
        let mut graph = Graph::new();

        let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();
        for (name, _) in self.blocks.iter() {
            let index = graph.add_node(name.clone());
            node_indices.insert(name.clone(), index);
        }

        for (producer_name, producer_block) in self.blocks.iter() {
            producer_block
                .registered_outputs
                .iter()
                .for_each(|(_, signal)| {
                    for (consumer_name, consumer_block) in self.blocks.iter() {
                        if consumer_block.registered_inputs.contains_key(signal) {
                            let delay = self.blocks.get(consumer_name).unwrap().block.delay();
                            if delay == 0 || cyclic_edges {
                                graph.add_edge(
                                    *node_indices.get(producer_name).unwrap(),
                                    *node_indices.get(consumer_name).unwrap(),
                                    signal.clone(),
                                );
                            }
                        }
                    }
                });
        }

        graph
    }
}
