extern crate wasm_bindgen;

use std::mem;

use wasm_bindgen::prelude::*;
use crate::cpu;

#[wasm_bindgen]
pub fn next_frame() -> *const bool {
    cpu::get_pointer_to_gfx()
}