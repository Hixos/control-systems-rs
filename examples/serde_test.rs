use std::fs::File;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Params {
    a: i32,
    b: f32,
}

impl Default for Params {
    fn default() -> Self {
        Params { a: 11, b: 22f32 }
    }
}
struct Block {
    p: Params,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            p: Params::default(),
        }
    }
}

impl Block {
    fn serialize_params(&self) -> Option<Value> {
        Some(serde_yaml::to_value(&self.p).unwrap())
    }

    fn deserialize_params(&mut self, value: Value) -> Result<()>{
        self.p = serde_yaml::from_value(value)?;
        Ok(())
    }
}   

fn main() {
    let p = Params { a: 11, b: 22f32 };

    let path = "sample_file.yaml";
    let file = File::open(path).expect("File should exist");

    let value: Value = serde_yaml::from_reader(file).unwrap();
    let pv = value.get("Params").unwrap();

    // serde_yaml::
    let p2: Params = serde_yaml::from_value(pv.to_owned()).unwrap();
    println!("Value: {:?}", p2);

    // let file = File::create(path).expect("File should exist");
    // serde_yaml::to_writer(file, &p).unwrap();
}

// use std::{fs::File, io};

// use serde_yaml::Value;

// fn main() -> io::Result<()> {
//     let path = "sample_file.yaml";
//     let file = File::open(path).expect("File should exist");

//     let mut value: Value = serde_yaml::from_reader(file).unwrap();

//     value["window"]["opacity"] = 3.0.into();

//     let file = File::create(path).expect("File should exist");
//     serde_yaml::to_writer(file, &value).unwrap();

//     Ok(())
// }
