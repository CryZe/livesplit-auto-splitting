extern crate asl_lang;

use std::fs;

fn main() {
    let script = fs::read_to_string("script.asl").unwrap();
    asl_lang::compile(&script).unwrap();
}
