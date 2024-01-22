use control_system::{
    io::Output, Block, BlockIO, ParameterStore, ParameterStoreError, Result, StepInfo, StepResult,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
