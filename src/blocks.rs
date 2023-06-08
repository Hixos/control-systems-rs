use crate::{ControlBlock, InputConnector, Interconnector, OutputConnector, StepInfo};
use anyhow::{anyhow, Result};
use num_traits::{Num, NumAssignOps};
use rbl_circular_buffer::CircularBuffer;

pub struct Add<T, const N: usize> {
    name: String,
    mul: [T; N],
    u: Vec<InputConnector<T>>,
    y: OutputConnector<T>,
}

impl<T, const N: usize> Add<T, N>
where
    T: Copy + Num + NumAssignOps,
{
    pub fn new(block_name: &str, u_names: &[&str; N], mul: Option<&[T; N]>, y_name: &str) -> Self {
        assert!(N > 0, "N must be greater than 0!");

        // By default leave inputs unchanged
        let def = [num_traits::one::<T>(); N];

        let mul = mul.unwrap_or(&def);

        Add {
            name: block_name.to_string(),
            mul: mul.to_owned(),
            u: u_names
                .iter()
                .map(|n| InputConnector::<T>::new(n))
                .collect(),
            y: OutputConnector::new(y_name),
        }
    }
}

impl<T, const N: usize> ControlBlock for Add<T, N>
where
    T: Copy + Num + NumAssignOps + 'static,
{
    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        self.u.iter_mut().try_for_each(|u| -> Result<()> {
            interconnector.register_input(u)?;
            Ok(())
        })
    }

    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_output(&mut self.y)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn step(&mut self, k: StepInfo) -> Result<()> {
        let mut sum = num_traits::zero::<T>();

        for (u, &m) in self.u.iter().zip(self.mul.iter()) {
            sum += m * u.input().ok_or(anyhow!(format!(
                "No input provided for '{}' in block '{}'",
                u.name(),
                self.name()
            )))?;
        }

        self.y.output(sum);
        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct Constant<T> {
    name: String,
    o1: OutputConnector<T>,
    value: T,
}

impl<T: Copy> Constant<T> {
    pub fn new(block_name: &str, value: T, out_name: &str) -> Self {
        Constant {
            name: block_name.to_string(),
            o1: OutputConnector::new(out_name),
            value,
        }
    }
}

impl<T: Copy + Num + 'static> ControlBlock for Constant<T> {
    #[allow(unused_variables)]
    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        Ok(())
    }

    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_output(&mut self.o1)?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn step(&mut self, k: StepInfo) -> Result<()> {
        self.o1.output(self.value);

        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct Delay<T, const D: usize> {
    name: String,
    i: InputConnector<T>,
    o: OutputConnector<T>,
    initial_value: T,
    buffer: CircularBuffer<T>,
}

impl<T: Copy, const D: usize> Delay<T, D> {
    pub fn new(block_name: &str, initial_value: T, i_name: &str, o_name: &str) -> Self {
        assert!(D > 0, "Delay must be > 0!");

        let mut instance = Delay {
            name: block_name.to_string(),
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
    fn register_inputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_input(&mut self.i)?;
        Ok(())
    }

    fn register_outputs(&mut self, interconnector: &mut Interconnector) -> Result<()> {
        interconnector.register_output(&mut self.o)?;
        Ok(())
    }

    fn step(&mut self, k: StepInfo) -> Result<()> {
        // In this case, "input" always refers to the iteration "k-1" since a delay block will always be executed first
        if k.k < D {
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

    fn name(&self) -> String {
        self.name.clone()
    }
}
