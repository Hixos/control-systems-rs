use crate::{io::AnySignal, Result};
use std::collections::HashMap;

pub trait BlockIO {
    fn name(&self) -> String;

    fn input_signals(&mut self) -> HashMap<String, &mut Option<AnySignal>>;
    fn output_signals(&mut self) -> HashMap<String, &mut AnySignal>;
}

pub trait Block: BlockIO {
    /// Propagates the block forward by one step
    fn step(&mut self, k: StepInfo) -> Result<StepResult>;

    fn delay(&self) -> u32 {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Stop,
}

#[derive(Debug, Clone, Copy)]
pub struct StepInfo {
    pub k: usize,
    pub dt: f64,
    pub t: f64,
}

impl StepInfo {
    pub fn new(dt: f64) -> Self {
        StepInfo { k: 1, dt, t: 0.0 }
    }
}
