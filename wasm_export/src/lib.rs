use bin_img_conv::{Bin, BitDepth, ColorType, Img};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn bin_to_img(mut input: &[u8], bit_depth: u8, color_type: u8) -> Vec<u8> {
    let mut input = Bin::new(&mut input).unwrap();
    input.1 = BitDepth::from_u8(bit_depth).unwrap();
    input.2 = ColorType::from_u8(color_type).unwrap();

    let output = Img::try_from(input).unwrap();

    output.into()
}

#[wasm_bindgen]
pub fn img_to_bin(input: Vec<u8>) -> Vec<u8> {
    let input = Img::new(input);
    let output = Bin::try_from(input).unwrap();

    (&output).try_into().unwrap()
}
