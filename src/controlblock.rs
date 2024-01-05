use std::collections::HashMap;
use anyhow::Result;
use crate::io::AnySignal;

pub trait BlockIO {
    fn name(&self) -> String;

    fn input_signals(&mut self) -> HashMap<String, &mut Option<AnySignal>>;
    fn output_signals(&mut self) -> HashMap<String, &mut AnySignal>;
}

pub trait Block : BlockIO {
    fn step(&mut self);

    fn delay(&self) -> u32 {
        0
    }
}