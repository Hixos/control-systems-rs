use anyhow::{anyhow, Result};
use crate::{ControlBlock, Interconnector, InputConnector, OutputConnector};
use num_traits::Num;
use rbl_circular_buffer::CircularBuffer;

pub struct Add<T> {
    i1: InputConnector<T>,
    i2: InputConnector<T>,
    o1: OutputConnector<T>,
}

impl<T: Copy> Add<T> {
    pub fn new(a_name: &str, b_name: &str, sum_name: &str) -> Self {
        Add {
            i1: InputConnector::new(a_name),
            i2: InputConnector::new(b_name),
            o1: OutputConnector::new(sum_name),
        }
    }
}

impl<T: Copy + Num + 'static> ControlBlock for Add<T> {
    fn notify_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i1)?;
        interconnector.register_input(&mut self.i2)?;
        Ok(())
    }

    fn notify_outputs(
        &mut self,
        interconnector: &mut Interconnector,
    ) -> Result<()> {
        interconnector.register_output(&mut self.o1)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn step(&mut self, k: usize) -> Result<()> {
        let a = self.i1.input().ok_or(anyhow!("Input 'a' not provided!"))?;
        let b = self.i2.input().ok_or(anyhow!("Input 'b' not provided!"))?;

        self.o1.output(a + b);

        Ok(())
    }
}

pub struct Constant<T> {
    o1: OutputConnector<T>,
    value: T,
}

impl<T: Copy> Constant<T> {
    pub fn new(value: T, out_name: &str) -> Self {
        Constant {
            o1: OutputConnector::new(out_name),
            value,
        }
    }
}

impl<T: Copy + Num + 'static> ControlBlock for Constant<T> {
    #[allow(unused_variables)]
    fn notify_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        Ok(())
    }

    fn notify_outputs(
        &mut self,
        interconnector: &mut Interconnector,
    ) -> Result<()> {
        interconnector.register_output(&mut self.o1)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn step(&mut self, k: usize) -> Result<()> {
        self.o1.output(self.value);

        Ok(())
    }
}

pub struct Delay<T, const D: usize> {
    i: InputConnector<T>,
    o: OutputConnector<T>,
    initial_value: T,
    buffer: CircularBuffer<T>,
}

impl<T: Copy, const D: usize> Delay<T, D> {
    pub fn new(initial_value: T, i_name: &str, o_name: &str) -> Self {
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
    fn notify_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i)?;
        Ok(())
    }

    fn notify_outputs(
        &mut self,
        interconnector: &mut Interconnector,
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