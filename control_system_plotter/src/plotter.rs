use std::error::Error;

use control_system::{io::Input, Block, ControlSystemError, StepResult};
use control_system::{BlockIO, ControlSystemBuilder, StepInfo};
use rust_data_inspector_signals::{PlotSampleSender, PlotSignalSample, PlotSignals};

use crate::AsF64Signals;
use control_system_lib::Result;

#[derive(BlockIO)]
pub struct Plotter<T> {
    #[blockio(block_name)]
    name: String,

    #[blockio(input)]
    u: Input<T>,

    senders: Vec<PlotSampleSender>,
}

impl<T: AsF64Signals + Default> Plotter<T> {
    pub fn new(name: &str, topic: &str, signals: &mut PlotSignals) -> Result<Self> {
        let names = T::names();

        let senders = names
            .iter()
            .map(|n| {
                signals
                    .add_signal(&format!("{topic}{n}"))
                    .map(|(_, sender)| sender)
                    .map_err(ControlSystemError::from_boxed)
            })
            .collect::<Result<Vec<PlotSampleSender>>>()?;

        Ok(Plotter {
            name: name.to_string(),
            u: Input::default(),
            senders,
        })
    }
}

impl<T: Clone + AsF64Signals + 'static> Block for Plotter<T> {
    fn step(&mut self, k: StepInfo) -> Result<StepResult, ControlSystemError> {
        // self.u.get().for(k.t, &mut self.senders);
        let sig = self.u.get();
        for (i, v) in sig.values().iter().enumerate() {
            self.senders[i].send(PlotSignalSample {
                time: k.t,
                value: *v,
            });
        }

        Ok(StepResult::Continue)
    }
}

pub fn add_plotter<T>(
    signal_name: &str,
    builder: &mut ControlSystemBuilder,
    signals: &mut PlotSignals,
) -> control_system_lib::Result<()>
where
    T: AsF64Signals + Default + Clone + 'static,
{
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let name = format!("plotter{}_{}", signal_name.replace('/', "_"), rand_string);
    let plotter = Plotter::<T>::new(name.as_str(), signal_name, signals)?;

    builder.add_block(plotter, &[("u", signal_name)], &[])?;

    Ok(())
}
