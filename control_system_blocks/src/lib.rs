use arrayinit::arr;
use control_system::{
    io::{Input, Output},
    Block, BlockIO, ParameterStore, ParameterStoreError, Result, StepInfo, StepResult,
};
use control_system_derive::BlockIO;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
pub struct ConstantParams<T> {
    pub c: T,
}

impl<T> From<T> for ConstantParams<T> {
    fn from(value: T) -> Self {
        ConstantParams { c: value }
    }
}

#[derive(BlockIO)]
pub struct Constant<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(output)]
    y: Output<T>,

    params: ConstantParams<T>,
}

impl<T> Constant<T>
where
    T: Default,
    Output<T>: Default,
{
    pub fn new(name: &str, params: ConstantParams<T>) -> Self {
        Constant {
            name: name.to_string(),
            y: Output::default(),
            params,
        }
    }
}

impl<T> Constant<T>
where
    T: Default + Serialize + DeserializeOwned + 'static,
{
    pub fn from_store(
        name: &str,
        store: &mut ParameterStore,
        default_params: ConstantParams<T>,
    ) -> Result<Self, ParameterStoreError> {
        let params = store.get_block_params(name, default_params)?;

        Ok(Constant::new(name, params))
    }
}

impl<T> Block for Constant<T>
where
    T: 'static + Clone,
{
    fn step(&mut self, _: StepInfo) -> Result<StepResult> {
        self.y.set(self.params.c.clone());
        Ok(StepResult::Continue)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DelayParameters<T> {
    pub initial_values: Vec<T>,
}

impl<T> From<Vec<T>> for DelayParameters<T> {
    fn from(value: Vec<T>) -> Self {
        DelayParameters {
            initial_values: value,
        }
    }
}

#[derive(BlockIO)]
pub struct Delay<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<T>,

    #[blockio(output)]
    y: Output<T>,

    buffer: Vec<T>,
    index: usize,
}

impl<T> Delay<T>
where
    T: Default + 'static,
{
    pub fn new(name: &str, params: DelayParameters<T>) -> Self {
        Delay {
            name: name.to_string(),
            u: Input::default(),
            y: Output::default(),
            buffer: params.initial_values,
            index: 0,
        }
    }
}

impl<T> Delay<T>
where
    T: Default + Serialize + DeserializeOwned + 'static,
{
    pub fn from_store(
        name: &str,
        store: &mut ParameterStore,
        default_params: DelayParameters<T>,
    ) -> Result<Self, ParameterStoreError> {
        let params = store.get_block_params(name, default_params)?;

        Ok(Self::new(name, params))
    }
}

impl<T> Block for Delay<T>
where
    T: 'static + Clone,
{
    fn step(&mut self, k: StepInfo) -> Result<StepResult> {
        let delay = self.delay() as usize;

        if k.k > 1 {
            let ix = (self.index + delay + 1) % delay; // index - 1
            self.buffer[ix] = self.u.get();
        }

        let v: T = self.buffer[self.index].clone();
        self.y.set(v);

        self.index = (self.index + 1) % delay;

        Ok(StepResult::Continue)
    }

    fn delay(&self) -> u32 {
        self.buffer.len() as u32
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
