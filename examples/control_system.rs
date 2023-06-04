use std::fmt::Display;

use anyhow::{anyhow, Result};
use control_system::{ControlBlock, ControlSystemBuilder, InputConnector, OutputConnector};
use num_traits::Num;
use rbl_circular_buffer::CircularBuffer;

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

    fn step(&mut self, k: usize) -> Result<()> {
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

    fn step(&mut self, k: usize) -> Result<()> {
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

    fn step(&mut self, k: usize) -> Result<()> {
        println!("Output: {}", self.i1.input().unwrap());
        Ok(())
    }
}

struct Delay<T, const D: usize> {
    i: InputConnector<T>,
    o: OutputConnector<T>,
    initial_value: T,
    buffer: CircularBuffer<T>,
}

impl<T: Copy, const D: usize> Delay<T, D> {
    fn new(initial_value: T, i_name: &str, o_name: &str) -> Self {
        assert!(D > 0, "Delay must be > 0!");

        let mut instance = Delay {
            i: InputConnector::new(i_name),
            o: OutputConnector::new(o_name),
            initial_value,
            buffer: CircularBuffer::new(D as usize),
        };

        // Prefill buffer with (delay - 1) samples
        for _ in 0usize..(D - 1usize) {
            instance.buffer.push(initial_value);
        }

        instance
    }
}

impl<T: Copy + 'static, const D: usize> ControlBlock for Delay<T, D> {
    fn notify_inputs(&mut self, interconnector: &mut control_system::Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i)?;
        Ok(())
    }

    fn notify_outputs(
        &mut self,
        interconnector: &mut control_system::Interconnector,
    ) -> Result<()> {
        interconnector.register_output(&mut self.o)?;
        Ok(())
    }

    fn step(&mut self, k: usize) -> Result<()> {
        // In this case, "input" always refers to the iteration "k-1" since a delay block will always be executed first
        if k < D {
            self.buffer.push(self.initial_value);
        } else {
            self.buffer.push(self.i.input().unwrap());
        }

        self.o.output(self.buffer.next().unwrap());

        Ok(())
    }

    fn delay(&self) -> usize {
        D
    }
}

fn main() -> Result<()> {
    //  c1 -- ADD ---------- PRINT
    //        |          |
    //        \- (z^-1) -/

    let delay = Delay::<i32, 1>::new(1, "sum", "a2");
    let c1 = Constant::new(1, "a1");

    let add = Add::<i32>::new("a1", "a2", "sum");

    let print = Print::<i32>::new("sum");

    let mut builder = ControlSystemBuilder::new();

    builder.add_block(delay)?;
    builder.add_block(c1)?;
    builder.add_block(add)?;
    builder.add_block(print)?;

    let mut control_system = builder.build()?;

    for k in 0..9 {
        control_system.step(k)?; // Prints 2,3,4,...,10
    }

    Ok(())
}
