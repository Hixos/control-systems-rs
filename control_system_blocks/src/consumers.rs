use control_system::{io::Input, Block, BlockIO, Result, StepInfo, StepResult};

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
