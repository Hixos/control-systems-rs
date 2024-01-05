use std::collections::HashMap;

use anyhow::Result;
use arrayinit::arr;
use blockio_derive::BlockIO;
use control_system::blocks::{Add, Constant, Print};
use control_system::io::{Input, Output};
use control_system::ControlSystemBuilder;
use control_system::{Block, BlockIO};

#[derive(BlockIO)]
struct Add1<const N: usize, T> where T: 'static {
    #[blockio(block_name)]
    name: String,

    #[blockio(input_arr)]
    u: [Input<T>; N],

    #[blockio(output)]
    y: Output<T>,
}

impl<const N: usize, T> Add1<N, T>
where
    T: Default,
{
    fn new(name: &str) -> Self {
        Add1 {
            name: name.to_string(),
            u: arr![|_| Input::<T>::default()],
            y: Output::<T>::default(),
        }
    }
}

impl<const N: usize, T> Block for Add1<N, T>
where
    T: Clone + std::iter::Sum,
{
    fn step(&mut self) {
        self.y.set(self.u.iter().map(|i| i.get()).sum());
    }
}

fn main() -> Result<()> {
    let add = Add::new("add");
    let mut add1 = Add1::<3, i32>::new("add1");
    println!("{:?}", add1.input_signals());

    let print = Print::new("print");

    let c1 = Constant::new("const_1", 3);
    let c2 = Constant::new("const_2", 4);

    let mut builder = ControlSystemBuilder::default();

    builder.add_block(c1, &[], &[("y", "c1")])?;
    builder.add_block(c2, &[], &[("y", "c2")])?;
    builder.add_block(add, &[("u1", "c1"), ("u2", "c2")], &[("y", "sum")])?;
    builder.add_block(
        add1,
        &[("u1", "sum"), ("u2", "c1"), ("u3", "c2")],
        &[("y", "sum1")],
    )?;
    builder.add_block(print, &[("u", "sum1")], &[])?;

    let mut controlsystem = builder.build()?;

    controlsystem.step();

    Ok(())
}
