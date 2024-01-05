use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use petgraph::{algo::toposort, dot::Dot, graph, prelude::NodeIndex, Graph};

use crate::{controlblock::Block, io::AnySignal};

pub struct ControlSystem {
    signals: HashMap<String, AnySignal>,
    blocks: Vec<Box<dyn Block>>,
    graph: Graph<String, String>,
}

impl ControlSystem {
    pub fn step(&mut self) {
        for b in self.blocks.iter_mut() {
            b.step();
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
    ) -> Result<&mut Self> {
        let name = block.name();
        
        if self.blocks.contains_key(&name) {
            return Err(anyhow!("A block named '{}' is already present!", name));
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

    pub fn build(mut self) -> Result<ControlSystem> {
        for (name, data) in self.blocks.iter_mut() {
            for (signal, input) in data.registered_inputs.iter() {
                let signal = self.signals.get(signal).ok_or(anyhow!(
                    "Could not connect input '{}/{}': No signal named '{}'",
                    name,
                    input,
                    signal
                ))?;

                data.block.connect_input(input, signal)?;
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
                Ok(ControlSystem {
                    signals: self.signals,
                    blocks,
                    graph,
                })
            }
            Err(cycle) => {
                Err(anyhow!(format!("Control system presents a cycle containing Node '{}'. You probably want to break the cycle by adding a delay block.", graph.node_weight(cycle.node_id()).unwrap())))
            }
        }
    }
}

impl ControlSystemBuilder {
    fn connect_outputs(
        &mut self,
        block_data: &mut BlockData,
        output_connections: &[(&str, &str)],
    ) -> Result<()> {
        let mut output_signals = block_data.block.output_signals();

        for (port, signal) in output_connections.iter() {
            if self.signals.contains_key(*signal) {
                // A signal with the same name is already produced by another output
                return Err(anyhow!(
                    "Cannot connect output '{}/{}': Signal '{}' already has a producer!",
                    block_data.block.name(),
                    port,
                    signal
                ));
            } else {
                self.signals.insert(
                    signal.to_string(),
                    output_signals
                        .get(*port)
                        .ok_or(anyhow!(
                            "No output port named '{}' in block '{}'",
                            port,
                            block_data.block.name()
                        ))?
                        .clone(),
                );
                block_data
                    .registered_outputs
                    .insert(port.to_string(), signal.to_string());
                output_signals.remove(*port);
            }
        }

        if output_signals.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "Output ports {:?} of block '{}' were not connected",
                output_signals.keys(),
                block_data.block.name()
            ))
        }
    }

    fn connect_inputs(
        &mut self,
        block_data: &mut BlockData,
        input_connections: &[(&str, &str)],
    ) -> Result<()> {
        let mut input_signals: HashSet<String> =
            block_data.block.input_signals().into_keys().collect();

        for (port, signal) in input_connections {
            if input_signals.contains(*port) {
                block_data
                    .registered_inputs
                    .insert(signal.to_string(), port.to_string());
                input_signals.remove(*port);
            } else {
                return Err(anyhow!(
                    "No input named {} in block {}",
                    port,
                    block_data.block.name()
                ));
            }
        }

        if input_signals.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "Input ports {:?} of block '{}' were not connected",
                input_signals.iter(),
                block_data.block.name()
            ))
        }
    }

    fn build_graph(&self, cyclic_edges: bool) -> Graph<String, String> {
        let mut graph = Graph::new();

        let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();
        for (name, block_data) in self.blocks.iter() {
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
