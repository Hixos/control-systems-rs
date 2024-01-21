use anyhow::Result;
use control_system::{ControlSystemBuilder, ControlSystemParameters};

use control_system_blocks::{Add, Constant, Delay, Print};

fn main() -> Result<()> {
    let add = Add::<i32, 2>::new("add");

    let print = Print::<i32>::new("print");

    let c1 = Constant::<i32>::new("const_1", 1.into());
    let delay = Delay::<i32>::new("delay", vec![0].into());

    let mut builder = ControlSystemBuilder::default();

    builder.add_block(c1, &[], &[("y", "one")])?;
    builder.add_block(delay, &[("u", "sum")], &[("y", "feedback")])?;

    builder.add_block(add, &[("u1", "one"), ("u2", "feedback")], &[("y", "sum")])?;
    builder.add_block(print, &[("u", "sum")], &[])?;

    let mut controlsystem = builder.build("adder", ControlSystemParameters::new(1.0))?;

    for _ in 0..10 {
        controlsystem.step()?;
    }

    Ok(())
}
