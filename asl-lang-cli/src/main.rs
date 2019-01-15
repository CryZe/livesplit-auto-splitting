extern crate asl_lang;

use std::fs;
use asl_lang::parity_wasm::serialize_to_file;

fn main() {
    let script = fs::read_to_string("script.asl").unwrap();
    let module = asl_lang::compile(&script).unwrap();
    serialize_to_file("out.wasm", module).unwrap();
}
