use arrayinit::arr;
use control_system::{
    io::{Input, Output},
    Block, BlockIO, Result, StepInfo, StepResult,
};
use control_system_derive::BlockIO;

#[derive(BlockIO)]
pub struct Add<T, const N: usize> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input_arr)]
    u: [Input<T>; N],

    #[blockio(output)]
    y: Output<T>,
}

impl<T, const N: usize> Add<T, N>
where
    T: Default,
    Output<T>: Default,
{
    pub fn new(name: &str) -> Self {
        Add {
            name: name.to_string(),
            u: arr![|_| Input::<T>::default()],
            y: Output::<T>::default(),
        }
    }
}

impl<T, const N: usize> Block for Add<T, N>
where
    T: Clone + std::iter::Sum + 'static,
{
    fn step(&mut self, _: StepInfo) -> Result<StepResult> {
        self.y.set(self.u.iter().map(|i| i.get()).sum());

        Ok(StepResult::Continue)
    }
}

#[derive(BlockIO)]
pub struct Constant<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(output)]
    y: Output<T>,

    value: T,
}

impl<T> Constant<T>
where
    T: Default,
    Output<T>: Default,
{
    pub fn new(name: &str, val: T) -> Self {
        Constant {
            name: name.to_string(),
            y: Output::default(),
            value: val,
        }
    }
}

impl<T> Block for Constant<T>
where
    T: 'static + Clone,
{
    fn step(&mut self, _: StepInfo) -> Result<StepResult> {
        self.y.set(self.value.clone());
        Ok(StepResult::Continue)
    }
}

#[derive(BlockIO)]
pub struct Delay<T, const D: usize> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<T>,

    #[blockio(output)]
    y: Output<T>,

    buffer: [T; D],
    index: usize,
}

impl<T, const D: usize> Delay<T, D>
where
    T: Default + 'static,
{
    pub fn new(name: &str, initial_value: [T; D]) -> Self {
        Delay {
            name: name.to_string(),
            u: Input::default(),
            y: Output::default(),
            buffer: initial_value,
            index: 0,
        }
    }
}

impl<T, const D: usize> Block for Delay<T, D>
where
    T: 'static + Clone,
{
    fn step(&mut self, k: StepInfo) -> Result<StepResult> {
        if k.k > 1 {
            let ix = (self.index + D + 1) % D; // index - 1
            self.buffer[ix] = self.u.get();
        }

        let v: T = self.buffer[self.index].clone();
        self.y.set(v);

        self.index = (self.index + 1) % D;

        Ok(StepResult::Continue)
    }

    fn delay(&self) -> u32 {
        D as u32
    }
}

#[derive(BlockIO)]
pub struct Generator<T, F> {
    #[blockio(block_name)]
    name: String,

    #[blockio(output)]
    y: Output<T>,

    generator: F,
}

impl<T, F> Generator<T, F>
where
    T: Default,
    Output<T>: Default,
{
    pub fn new(name: &str, generator: F) -> Self {
        Generator {
            name: name.to_string(),
            y: Output::default(),
            generator,
        }
    }
}

impl<T, F> Block for Generator<T, F>
where
    T: 'static + Clone,
    F: Fn() -> T,
{
    fn step(&mut self, _: StepInfo) -> Result<StepResult> {
        self.y.set((self.generator)());
        Ok(StepResult::Continue)
    }
}

#[derive(BlockIO)]
pub struct Print<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<T>,
}

impl<T> Print<T>
where
    T: Default + 'static,
{
    pub fn new(name: &str) -> Self {
        Print {
            name: name.to_string(),
            u: Input::default(),
        }
    }
}

impl<T> Block for Print<T>
where
    T: core::fmt::Debug + Clone + 'static,
{
    fn step(&mut self, k: StepInfo) -> Result<StepResult> {
        println!(
            "t: {:.2} {}->{} = {:?}",
            k.t,
            self.name,
            self.u.signal_name(),
            self.u.get()
        );
        Ok(StepResult::Continue)
    }
}
