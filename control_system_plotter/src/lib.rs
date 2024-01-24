extern crate control_system_lib as control_system;
mod plotter;
pub use plotter::{add_plotter, Plotter};

use num_traits::Num;

pub trait AsF64Signals {
    fn names() -> Vec<String>;

    fn values(&self) -> Vec<f64>;
}

impl<T: Num + Into<f64> + Copy> AsF64Signals for T {
    fn names() -> Vec<String> {
        vec!["".to_string()]
    }

    fn values(&self) -> Vec<f64> {
        vec![(*self).into()]
    }
}
