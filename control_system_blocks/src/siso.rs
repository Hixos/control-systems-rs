use control_system::{
    io::{Input, Output},
    Block, BlockIO, ParameterStore, ParameterStoreError, StepInfo, StepResult, Result
};
use num::{zero, Float, FromPrimitive};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

impl<T: Clone, const N: usize> From<[T; N]> for DelayParameters<T> {
    fn from(value: [T; N]) -> Self {
        DelayParameters {
            initial_values: value.to_vec(),
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




#[derive(Serialize, Deserialize)]
pub struct PIDParams<T> {
    pub kp: T,
    pub ki: T,
    pub kd: T,

    pub acc0: T,
}
impl<T> Default for PIDParams<T>
where
    T: Float,
{
    fn default() -> Self {
        PIDParams {
            kp: zero(),
            ki: zero(),
            kd: zero(),
            acc0: zero(),
        }
    }
}

#[derive(BlockIO)]
pub struct PID<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<T>,

    #[blockio(output)]
    y: Output<T>,

    params: PIDParams<T>,

    acc: T,
    last_err: T,
}

impl<T> PID<T>
where
    T: Float,
    Input<T>: Default,
    Output<T>: Default,
{
    pub fn new(name: &str, params: PIDParams<T>) -> Self {
        PID {
            name: name.to_string(),
            u: Input::default(),
            y: Output::default(),
            acc: params.acc0,
            last_err: zero(),
            params,
        }
    }
}

impl<T> PID<T>
where
    T: Float + Serialize + DeserializeOwned + 'static,
    Input<T>: Default,
{
    pub fn from_store(
        name: &str,
        store: &mut ParameterStore,
        default_params: PIDParams<T>,
    ) -> Result<Self, ParameterStoreError> {
        let params = store.get_block_params(name, default_params)?;

        Ok(PID::new(name, params))
    }
}

impl<T> Block for PID<T>
where
    T: Float + FromPrimitive + 'static + Clone,
{
    fn step(&mut self, stepinfo: StepInfo) -> Result<StepResult> {
        let dt: T = FromPrimitive::from_f64(stepinfo.dt).unwrap();

        let err = self.u.get();
        let der = (err - self.last_err) / dt;
        let int = self.acc + err * dt;

        self.y
            .set(err * self.params.kp + der * self.params.kd + int * self.params.ki);

        self.last_err = err;
        self.acc = int;

        Ok(StepResult::Continue)
    }
}
