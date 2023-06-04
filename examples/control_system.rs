use std::fmt::Display;

use anyhow::Result;
use control_system::{blocks, ControlBlock, ControlSystemBuilder, InputConnector};

struct Print<T> {
    i1: InputConnector<T>,
}

impl<T: Copy> Print<T> {
    fn new(in_name: &str) -> Self {
        Print {
            i1: InputConnector::new(in_name),
        }
    }
}

impl<T: Copy + Display + 'static> ControlBlock for Print<T> {
    fn notify_inputs(&mut self, interconnector: &mut control_system::Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i1)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn notify_outputs(
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
}

fn main() -> Result<()> {
    //  c1 -- ADD ---------- PRINT
    //        |          |
    //        \- (z^-1) -/

    let delay = blocks::Delay::<i32, 1>::new(0, "sum", "a2");
    let c1 = blocks::Constant::new(1, "a1");

    let add = blocks::Add::<i32>::new("a1", "a2", "sum");

    let print = Print::<i32>::new("sum");

    let mut builder = ControlSystemBuilder::new();

    builder.add_block(delay)?;
    builder.add_block(c1)?;
    builder.add_block(add)?;
    builder.add_block(print)?;

    let mut control_system = builder.build()?;

    for k in 0..10 {
        control_system.step(k)?; // Prints 1,2,3,...,10
    }

    Ok(())
}
