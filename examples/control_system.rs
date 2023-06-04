use std::fmt::Display;

use anyhow::Result;
use control_system::{blocks, ControlBlock, ControlSystemBuilder, InputConnector};

struct Print<T> {
    name: String,
    i1: InputConnector<T>,
}

impl<T: Copy> Print<T> {
    fn new(block_name: &str, in_name: &str) -> Self {
        Print {
            name: block_name.to_string(),
            i1: InputConnector::new(in_name),
        }
    }
}

impl<T: Copy + Display + 'static> ControlBlock for Print<T> {
    fn register_inputs(&mut self, interconnector: &mut control_system::Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i1)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_outputs(
        &mut self,
        interconnector: &mut control_system::Interconnector,
    ) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn step(&mut self, k: usize) -> Result<()> {
        println!("Output: {}", self.i1.input().unwrap());
        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

fn main() -> Result<()> {
    //  c1 -- ADD ---------- PRINT
    //        |          |
    //        \- (z^-1) -/

    let delay = blocks::Delay::<i32, 1>::new("delay", 0, "sum", "a2");
    let c1 = blocks::Constant::new("c1", 1, "a1");

    let add = blocks::Add::<i32>::new("add", "a1", "a2", "sum");

    let print = Print::<i32>::new("sum", "sum");

    let mut builder = ControlSystemBuilder::new();

    builder.add_block(c1)?;
    builder.add_block(add)?;
    builder.add_block(delay)?;
    builder.add_block(print)?;

    let mut control_system = builder.build()?;

    for k in 0..10 {
        control_system.step(k)?; // Prints 1,2,3,...,10
    }

    Ok(())
}
