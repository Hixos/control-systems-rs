use std::{fmt::Display, fs::File};

use anyhow::Result;
use control_system::{
    blocks, ControlBlock, ControlSystemBuilder, InputConnector, Prober, StepInfo,
};

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
    fn register_inputs(
        &mut self,
        interconnector: &mut control_system::Interconnector,
    ) -> Result<()> {
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
    fn step(&mut self, k: StepInfo) -> Result<()> {
        println!("Output: {}", self.i1.input().unwrap());
        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

struct Printer;

impl<T: Display> Prober<T> for Printer {
    fn probe(&self, signal: &str, v: Option<T>, k: StepInfo) {
        println!("{}[{}], val: {}", signal, k.k, v.unwrap());
    }
}

fn main() -> Result<()> {
    //  c1 -- ADD ---------- PRINT
    //        |          |
    //        \- (z^-1) -/

    let delay = blocks::Delay::<f32, 1>::new("delay", 0f32, "sum", "a2");
    let c1 = blocks::Constant::new("c1", 1f32, "a1");

    let add = blocks::Add::<f32, 2>::new("add", &["a1", "a2"], None, "sum");

    let print = Print::<f32>::new("sum", "sum");

    let mut builder = ControlSystemBuilder::new();

    builder.add_block(c1)?;
    builder.add_block(add)?;
    builder.add_block(delay)?;
    builder.add_block(print)?;

    builder.probe::<f32, _>("a1", Printer {})?;
    builder.fnprobe::<f32, _>("a2", |s, v, k| {
        println!("{}[{}], val: {} (closure)", s, k.k, v.unwrap());
    })?;

    let mut control_system = builder.build(0f64)?;

    let filename = "parameters.yaml";
    match File::open(filename) {
        Ok(file) => {
            control_system.load_params(file, true)?;
        }
        Err(_) => {} // Load default parameters
    }

    for _ in 0..10 {
        control_system.step(1f64)?; // Prints 1,2,3,...,10
    }

    let file = File::create(filename)?;
    control_system.save_params(file)?;

    Ok(())
}
