use crate::io::AnySignal;
use std::collections::HashMap;

pub trait BlockIO {
    fn name(&self) -> String;

    fn input_signals(&mut self) -> HashMap<String, &mut Option<AnySignal>>;
    fn output_signals(&mut self) -> HashMap<String, &mut AnySignal>;
}

pub trait Block: BlockIO {
    fn step(&mut self, k: StepInfo);

    fn delay(&self) -> u32 {
        0
    }
}