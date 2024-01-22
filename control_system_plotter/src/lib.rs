extern crate control_system_lib as control_system;
mod plotter;
pub use plotter::{add_plotter, Plotter};

use num_traits::Num;
use rust_data_inspector_signals::{
    PlotSampleSender, PlotSignalError, PlotSignalSample, PlotSignals,
};

pub fn register_signal_helper(
    topic: &str,
    name: &str,
    signals: &mut PlotSignals,
) -> Result<PlotSampleSender, PlotSignalError> {
    let signal_name = [topic, name].join("/");

    signals.add_signal(&signal_name).map(|(_, prod)| prod)
}

pub trait Plottable {
    fn register_signals(
        topic: &str,
        signals: &mut PlotSignals,
    ) -> Result<Vec<PlotSampleSender>, PlotSignalError>;

    fn plot_sample(&self, time: f64, producers: &mut Vec<PlotSampleSender>);
}

impl<T: Num + Into<f64> + Copy> Plottable for T {
    fn plot_sample(&self, time: f64, producers: &mut Vec<PlotSampleSender>) {
        producers[0]
            .send(PlotSignalSample {
                time,
                value: (*self).into(),
            })
            .expect("Error sending signal");
    }

    fn register_signals(
        topic: &str,
        signals: &mut PlotSignals,
    ) -> Result<Vec<PlotSampleSender>, PlotSignalError> {
        let prod = signals.add_signal(topic).map(|(_, prod)| prod)?;

        Ok(vec![prod])
    }
}
