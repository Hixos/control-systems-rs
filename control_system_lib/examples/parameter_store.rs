use std::path::Path;

use control_system_lib::ParameterStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TestConfig1 {
    a: i32,
    b: String,
    c: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TestConfig2 {
    x: TestConfig1,
    y: Option<bool>,
}

fn main() {
    let tc1_default = TestConfig1 {
        a: 23,
        b: "Hello".to_string(),
        c: 123.456,
    };

    let tc2_default = TestConfig2 {
        x: TestConfig1 {
            a: 69,
            b: "WooWeee".to_string(),
            c: 420.69,
        },
        y: Some(false),
    };

    let mut store = ParameterStore::new(Path::new("test.toml"), "test_cs").unwrap();

    let tc1 = store.get_block_params("block_1", tc1_default).unwrap();
    let tc2 = store.get_block_params("block_2", tc2_default).unwrap();

    println!("{:?}", tc1);
    println!("{:?}", tc2);

    store.save().unwrap();
}
