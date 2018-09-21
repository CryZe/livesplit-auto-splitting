extern crate asl_runtime;

use asl_runtime::Runtime;
use std::error::Error;
use std::time::Duration;
use std::{fs, thread};

fn main() -> Result<(), Box<Error>> {
    let buffer = fs::read("asl-language/out.wasm")?;
    // let buffer = fs::read("asl-rust-example/target/wasm32-unknown-unknown/release/asl.wasm")?;
    let mut runtime = Runtime::new(&buffer)?;
    loop {
        thread::sleep(Duration::from_millis(16));
        if let Some(action) = runtime.step()? {
            eprintln!("{:?}", action);
        }
    }

    // Ok(())
}
