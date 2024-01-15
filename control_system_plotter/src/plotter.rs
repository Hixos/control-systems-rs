use control_system::{io::Input, Block};
use control_system_derive::BlockIO;
use control_system::BlockIO;
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
    fn step(&mut self, k: control_system::controlblock::StepInfo) {
        self.u.get().plot_sample(k.t, &mut self.producers);
    }
}
