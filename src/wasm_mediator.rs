extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use crate::cpu::Chip8;

#[wasm_bindgen]
pub fn next_frame() -> Vec<i32> {
    let chip8 = Chip8::initialize();

    let arr = [false; 2048];

    arr.to_vec().iter().map(|i| match i { true => 1, false => 0 }).collect()
}