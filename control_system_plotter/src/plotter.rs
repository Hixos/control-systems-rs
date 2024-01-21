use control_system::{StepResult, io::Input, Block, ControlSystemError};
use control_system::{BlockIO, ControlSystemBuilder, StepInfo};
use rust_data_inspector_signals::{PlotSignalProducer, PlotSignals};

use crate::Plottable;

#[derive(BlockIO)]
pub struct Plotter<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<T>,

    producers: Vec<PlotSignalProducer>,
}

impl<T: Plottable + Default> Plotter<T> {
    pub fn new(name: &str, topic: &str, signals: &mut PlotSignals) -> Self {
        Plotter {
            name: name.to_string(),
            u: Input::default(),
            producers: T::register_signals(topic, signals).expect("Error registering signal"),
        }
    }
}

impl<T: Clone + Plottable + 'static> Block for Plotter<T> {
    fn step(
        &mut self,
        k: StepInfo,
    ) -> Result<StepResult, ControlSystemError> {
        self.u.get().plot_sample(k.t, &mut self.producers);

        Ok(StepResult::Continue)
    }
}

pub fn add_plotter<T>(
    signal_name: &str,
    builder: &mut ControlSystemBuilder,
    signals: &mut PlotSignals,
) -> control_system::Result<()>
where
    T: Plottable + Default + Clone + 'static,
{
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let name = format!("plotter{}_{}", signal_name.replace('/', "_"), rand_string);
    let plotter = Plotter::<T>::new(name.as_str(), signal_name, signals);

    builder.add_block(plotter, &[("u", signal_name)], &[])?;

    Ok(())
}
