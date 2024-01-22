use anyhow::Result;
use control_system_blocks::{consumers::Print, math::Add, producers::Constant, siso::Delay};
use control_system_lib::{ControlSystemBuilder, ControlSystemParameters, StepResult};

fn main() -> Result<()> {
    let add = Add::<i32, 2>::new("add", [1, 1].into());

    let print = Print::<i32>::new("print");

    let c1 = Constant::<i32>::new("const_1", 1.into());
    let delay = Delay::<i32>::new("delay", vec![0].into());

    let mut builder = ControlSystemBuilder::default();

    builder.add_block(c1, &[], &[("y", "one")])?;
    builder.add_block(delay, &[("u", "sum")], &[("y", "feedback")])?;

    builder.add_block(add, &[("u1", "one"), ("u2", "feedback")], &[("y", "sum")])?;
    builder.add_block(print, &[("u", "sum")], &[])?;

    let mut controlsystem = builder.build(
        "adder",
        ControlSystemParameters {
            dt: 1.0,
            max_iter: 10,
        },
    )?;

    while controlsystem.step()? != StepResult::Stop {}

    Ok(())
}
