use std::collections::HashMap;

use crate::{
    controlblock::{Block, BlockIO},
    io::{AnySignal, Input, Output},
};
use anyhow::{anyhow, Result};

pub struct Constant {
    name: String,
    y: Output<i32>,
    value: i32,
}
impl Constant {
    pub fn new(name: &str, val: i32) -> Self {
        Constant {
            name: name.to_string(),
            y: Output::default(),
            value: val,
        }
    }
}
impl BlockIO for Constant {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn connect_input(&mut self, _name: &str, _signal: &AnySignal) -> Result<()> {
        #![allow(clippy::match_single_binding)]
        match _name {
            _ => Err(anyhow!("No input named {} in block {}", _name, self.name())),
        }
    }

    fn input_signals(&self) -> std::collections::HashMap<String, Option<AnySignal>> {
        #![allow(unused_mut, clippy::let_and_return)]
        let mut hm = HashMap::new();
        hm
    }

    fn output_signals(&self) -> HashMap<String, AnySignal> {
        #![allow(unused_mut)]
        let mut hm = HashMap::new();
        hm.insert("y".to_string(), self.y.get_signal());
        hm
    }
}

impl Block for Constant {
    fn step(&mut self) {
        self.y.set(self.value);
    }
}

pub struct Add {
    name: String,
    u1: Input<i32>,
    u2: Input<i32>,
    y: Output<i32>,
}

impl Add {
    pub fn new(name: &str) -> Self {
        Add {
            name: name.to_string(),
            u1: Input::default(),
            u2: Input::default(),
            y: Output::default(),
        }
    }
}

impl BlockIO for Add {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn connect_input(&mut self, _name: &str, _signal: &AnySignal) -> Result<()> {
        #![allow(clippy::match_single_binding)]
        match _name {
            "u1" => self.u1.connect(_signal),
            "u2" => self.u2.connect(_signal),
            _ => Err(anyhow!("No input named {} in block {}", _name, self.name())),
        }
    }

    fn input_signals(&self) -> std::collections::HashMap<String, Option<AnySignal>> {
        #![allow(unused_mut)]
        let mut hm = HashMap::new();
        hm.insert("u1".to_string(), self.u1.get_signal());
        hm.insert("u2".to_string(), self.u2.get_signal());
        hm
    }

    fn output_signals(&self) -> HashMap<String, AnySignal> {
        #![allow(unused_mut)]
        let mut hm = HashMap::new();
        hm.insert("y".to_string(), self.y.get_signal());
        hm
    }
}

impl Block for Add {
    fn step(&mut self) {
        self.y.set(self.u1.get() + self.u2.get());
    }
}

pub struct Print {
    name: String,
    u: Input<i32>,
}

impl Print {
    pub fn new(name: &str) -> Self {
        Print {
            name: name.to_string(),
            u: Input::default(),
        }
    }
}


impl BlockIO for Print {
    fn name(&self) -> String {
        "print".to_string()
    }

    fn connect_input(&mut self, _name: &str, _signal: &AnySignal) -> Result<()> {
        #![allow(clippy::match_single_binding)]
        match _name {
            "u" => self.u.connect(_signal),
            _ => Err(anyhow!("No input named {} in block {}", _name, self.name())),
        }
    }

    fn input_signals(&self) -> std::collections::HashMap<String, Option<AnySignal>> {
        #![allow(unused_mut)]
        let mut hm = HashMap::new();
        hm.insert("u".to_string(), self.u.get_signal());
        hm
    }

    fn output_signals(&self) -> HashMap<String, AnySignal> {
        #![allow(unused_mut)]
        let mut hm = HashMap::new();
        hm
    }
}

impl Block for Print {
    fn step(&mut self) {
        println!("{}", self.u.get());
    }
}
