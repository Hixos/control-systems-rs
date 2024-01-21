use arrayinit::arr;
use control_system::{
    io::{Input, Output},
    Block, BlockIO, ParameterStore, Result, StepInfo, StepResult,
};
use num::Num;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddParams<T> {
    gains: Vec<T>,
}

impl<T> From<Vec<T>> for AddParams<T> {
    fn from(value: Vec<T>) -> Self {
        AddParams { gains: value }
    }
}

impl<T, const N: usize> From<[T; N]> for AddParams<T>
where
    T: Clone,
{
    fn from(value: [T; N]) -> Self {
        Self {
            gains: value.to_vec(),
        }
    }
}

#[derive(BlockIO)]
pub struct Add<T, const N: usize> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input_arr)]
    u: [Input<T>; N],

    #[blockio(output)]
    y: Output<T>,

    params: AddParams<T>,
}

impl<T, const N: usize> Add<T, N>
where
    T: Default,
    Output<T>: Default,
{
    pub fn new(name: &str, params: AddParams<T>) -> Self {
        assert!(params.gains.len() == N);

        Add {
            name: name.to_string(),
            u: arr![|_| Input::<T>::default()],
            y: Output::<T>::default(),
            params,
        }
    }
}

impl<T, const N: usize> Add<T, N>
where
    T: Default + Serialize + DeserializeOwned,
    Output<T>: Default,
{
    pub fn from_store(
        name: &str,
        store: &mut ParameterStore,
        default: AddParams<T>,
    ) -> Result<Self> {
        let params = store.get_block_params(name, default)?;
        Ok(Add {
            name: name.to_string(),
            u: arr![|_| Input::<T>::default()],
            y: Output::<T>::default(),
            params,
        })
    }
}

impl<T, const N: usize> Block for Add<T, N>
where
    T: Clone + std::iter::Sum + 'static + Num,
{
    fn step(&mut self, _: StepInfo) -> Result<StepResult> {
        self.y.set(
            self.u
                .iter()
                .zip(self.params.gains.iter())
                .map(|(i, k)| i.get() * k.clone())
                .sum(),
        );

        Ok(StepResult::Continue)
    }
}
