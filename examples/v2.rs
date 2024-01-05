use anyhow::Result;
use blockio_derive::BlockIO;
use control_system::blocks::{Add, Constant, Print};
use control_system::io::{Input, Output};
use control_system::ControlSystemBuilder;
use control_system::{Block, BlockIO};

#[derive(BlockIO)]
struct Add1 {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<i32>,

    #[blockio(output)]
    y: Output<i32>,
}

impl Add1 {
    fn new(name: &str) -> Self {
        Add1 {
            name: name.to_string(),
            u: Input::default(),
            y: Output::default(),
        }
    }
}

impl Block for Add1 {
    fn step(&mut self) {
        self.y.set(self.u.get() + 1);
    }
}

fn main() -> Result<()> {
    let add = Add::new("add");
    let add1 = Add1::new("add1");

    let print = Print::new("print");

    let c1 = Constant::new("const_1", 3);
    let c2 = Constant::new("const_2", 4);

    let mut builder = ControlSystemBuilder::default();

    builder.add_block(c1, &[], &[("y", "c1")])?;
    builder.add_block(c2, &[], &[("y", "c2")])?;
    builder.add_block(add, &[("u1", "c1"), ("u2", "c2")], &[("y", "sum")])?;
    builder.add_block(add1, &[("u", "sum")], &[("y", "sum1")])?;
    builder.add_block(print, &[("u", "sum1")], &[])?;

    let mut controlsystem = builder.build()?;

    controlsystem.step();

    Ok(())
}
