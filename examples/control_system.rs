use std::fmt::Display;

use anyhow::{anyhow, Result};
use control_system::{ControlBlock, InputConnector, OutputConnector, ControlSystemBuilder};
use num_traits::Num;

struct Add<T> {
    i1: InputConnector<T>,
    i2: InputConnector<T>,
    o1: OutputConnector<T>,
}

impl<T: Copy> Add<T> {
    fn new(a_name: &str, b_name: &str, sum_name: &str) -> Self {
        Add {
            i1: InputConnector::new(a_name),
            i2: InputConnector::new(b_name),
            o1: OutputConnector::new(sum_name),
        }
    }
}

impl<T: Copy + Num + 'static> ControlBlock for Add<T> {
    fn notify_inputs(&mut self, interconnector: &mut control_system::Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i1)?;
        interconnector.register_input(&mut self.i2)?;
        Ok(())
    }

    fn notify_outputs(
        &mut self,
        interconnector: &mut control_system::Interconnector,
    ) -> Result<()> {
        interconnector.register_output(&mut self.o1)?;
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        let a = self.i1.input().ok_or(anyhow!("Input 'a' not provided!"))?;
        let b = self.i2.input().ok_or(anyhow!("Input 'b' not provided!"))?;

        self.o1.output(a + b);

        Ok(())
    }
}

struct Constant<T> {
    o1: OutputConnector<T>,
    value: T,
}

impl<T: Copy> Constant<T> {
    fn new(value: T, out_name: &str) -> Self {
        Constant {
            o1: OutputConnector::new(out_name),
            value,
        }
    }
}

impl<T: Copy + Num + 'static> ControlBlock for Constant<T> {
    fn notify_inputs(&mut self, interconnector: &mut control_system::Interconnector) -> Result<()> {
        Ok(())
    }

    fn notify_outputs(
        &mut self,
        interconnector: &mut control_system::Interconnector,
    ) -> Result<()> {
        interconnector.register_output(&mut self.o1)?;
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        self.o1.output(self.value);

        Ok(())
    }
}

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

    fn notify_outputs(
        &mut self,
        interconnector: &mut control_system::Interconnector,
    ) -> Result<()> {
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        println!("Output: {}", self.i1.input().unwrap());
        Ok(())
    }
}

fn main() -> Result<()> {
    //  c1 --\
    //        \
    //         ADD---PRINT
    //        /
    //  c2 --/

    let c1 = Constant::new(33, "c1");
    let c2 = Constant::new(44, "c2");

    let add = Add::<i32>::new("c1", "c2", "sum");

    let print = Print::<i32>::new("sum");

    let mut builder = ControlSystemBuilder::new();

    builder.add_block(c1)?;
    builder.add_block(c2)?;
    builder.add_block(add)?;
    builder.add_block(print)?;

    let mut control_system = builder.build()?;

    control_system.step()?; // Prints 77

    Ok(())
}
