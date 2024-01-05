use std::collections::HashMap;
use anyhow::Result;
use crate::io::AnySignal;

pub trait BlockIO {
    fn name(&self) -> String;

    fn connect_input(&mut self, name: &str, signal: &AnySignal) -> Result<()>;

    fn input_signals(&self) -> HashMap<String, Option<AnySignal>>;
    fn output_signals(&self) -> HashMap<String, AnySignal>;
}

pub trait Block : BlockIO {
    fn step(&mut self);

    fn delay(&self) -> u32 {
        0
    }
}