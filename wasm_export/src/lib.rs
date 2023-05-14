use std::io::BufReader;

use bin_img_conv::{Bin, BitDepth, ColorType, Img, LowMemoryReadableVec};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

fn init() {
    std::panic::set_hook(Box::new(|e| {
        let trace = backtrace::Backtrace::new();
        alert(&format!("{}\n{:?}", e, trace));
    }));
}

#[wasm_bindgen]
pub fn bin_to_img(input: Vec<u8>, bit_depth: u8, color_type: u8) -> Vec<u8> {
    init();

    let mut input = Bin::new(&mut BufReader::new(LowMemoryReadableVec::from(input))).unwrap();
    input.1 = BitDepth::from_u8(bit_depth).unwrap();
    input.2 = ColorType::from_u8(color_type).unwrap();

    let output = Img::try_from(input).unwrap();

    output.into()
}

#[wasm_bindgen]
pub fn img_to_bin(input: Vec<u8>) -> Vec<u8> {
    init();

    let input = Img::new(input);
    let output = Bin::try_from(input).unwrap();

    (&output).try_into().unwrap()
}
