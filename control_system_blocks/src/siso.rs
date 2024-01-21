use control_system::{
    io::{Input, Output},
    Block, BlockIO, ParameterStore, ParameterStoreError, StepInfo, StepResult,
};
use num::{zero, Float, FromPrimitive};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
    fn step(&mut self, stepinfo: StepInfo) -> crate::Result<StepResult> {
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
