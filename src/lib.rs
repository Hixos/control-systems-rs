pub mod iochannel;
pub mod blocks;


use std::any::{Any, TypeId};
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use iochannel::{io_channel, IOChannel, InputReceiver, OutputSender};

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

struct ChannelEntry {
    tid: TypeId,
    has_producer: bool,
    channel: Box<dyn Any>,
}

struct ChannelBundle<T> {
    channel: IOChannel<T>,
    sender: Option<OutputSender<T>>,
}

pub struct Interconnector {
    channels: HashMap<String, ChannelEntry>,
}

impl Interconnector {
    fn new() -> Interconnector {
        Interconnector {
            channels: HashMap::new(),
        }
    }

    pub fn register_output<T: 'static>(&mut self, conn: &mut OutputConnector<T>) -> Result<()> {
        let connector_tid = TypeId::of::<T>();

        if !self.channels.contains_key(&conn.name) {
            self.insert_new_channel::<T>(conn.name.clone());
        }

        let channel = {
            let ChannelEntry {
                tid,
                has_producer,
                channel,
            } = self.channels.get_mut(&conn.name).unwrap();
            if *tid == connector_tid {
                let channel = channel.downcast_mut::<ChannelBundle<T>>().unwrap();

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
        let connector_tid = TypeId::of::<T>();

        if !self.channels.contains_key(&conn.name) {
            self.insert_new_channel::<T>(conn.name.clone());
        }

        let channel = {
            let ChannelEntry {
                tid,
                has_producer: _,
                channel,
            } = self.channels.get_mut(&conn.name).unwrap();
            if *tid == connector_tid {
                Ok(channel.downcast_mut::<ChannelBundle<T>>().unwrap())
            } else {
                Err(anyhow!(format!(
                    "Trying to re-register output singal '{}' with a different type ({:?} -> {:?})",
                    conn.name, tid, connector_tid
                )))
            }
        }?;

        conn.input = Some(channel.channel.new_input_receiver());

        Ok(())
    }

    fn insert_new_channel<T: 'static>(&mut self, name: String) {
        let connector_tid = TypeId::of::<T>();

        let (channel, sender) = io_channel();
        let bundle: Box<ChannelBundle<T>> = Box::new(ChannelBundle {
            channel,
            sender: Some(sender),
        });

        self.channels.insert(
            name,
            ChannelEntry {
                tid: connector_tid,
                has_producer: false,
                channel: bundle,
            },
        );
    }
}

pub struct ControlSystemBuilder {
    control_system: ControlSystem,
    interconnector: Interconnector,
}

impl ControlSystemBuilder {
    pub fn new() -> ControlSystemBuilder {
        ControlSystemBuilder {
            control_system: ControlSystem::new(),
            interconnector: Interconnector::new(),
        }
    }

    pub fn add_block<T: ControlBlock + 'static>(&mut self, mut block: T) -> Result<()> {
        block.notify_outputs(&mut self.interconnector)?;
        block.notify_inputs(&mut self.interconnector)?;

        self.control_system.blocks.push(Box::new(block));
        Ok(())
    }

    pub fn build(self) -> Result<ControlSystem> {
        Ok(self.control_system)
    }
}

pub trait ControlBlock {
    fn notify_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()>;

    fn notify_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()>;

    fn step(&mut self, k: usize) -> Result<()>;

    fn delay(&self) -> usize {
        0usize
    }
}

pub struct ControlSystem {
    blocks: Vec<Box<dyn ControlBlock>>,
}

impl ControlSystem {
    fn new() -> ControlSystem {
        ControlSystem { blocks: vec![] }
    }

    pub fn step(&mut self, k: usize) -> Result<()> {
        for block in self.blocks.iter_mut() {
            block.step(k)?;
        }

        Ok(())
    }
}
