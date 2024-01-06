use anyhow::Result;
use control_system::ControlSystemBuilder;
use control_system::{Block, BlockIO};

use control_system_blocks::{Add, Constant, Print};

fn main() -> Result<()> {
    let add = Add::<i32, 2>::new("add");

    let print = Print::<i32>::new("print");

    let c1 = Constant::<i32>::new("const_1", 3);
    let c2 = Constant::<i32>::new("const_2", 4);

    let mut builder = ControlSystemBuilder::default();

    builder.add_block(c1, &[], &[("y", "c1")])?;
    builder.add_block(c2, &[], &[("y", "c2")])?;
    builder.add_block(add, &[("u1", "c1"), ("u2", "c2")], &[("y", "sum")])?;
    builder.add_block(print, &[("u", "sum")], &[])?;

    let mut controlsystem = builder.build()?;

    controlsystem.step();

    Ok(())
}
