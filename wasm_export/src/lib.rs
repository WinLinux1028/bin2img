use std::{ffi::c_void, mem::size_of};

use bin_img_conv::{Bin, BitDepth, ColorType, Img, LowMemoryReadableVec};

extern "C" {
    fn malloc(_: usize) -> *mut c_void;
    fn free(_: *mut c_void);
}

#[no_mangle]
pub fn bin_to_img(
    input_buf: *mut u8,
    input_len: usize,
    bit_depth: u8,
    color_type: u8,
) -> *mut Vec<u8> {
    let input = unsafe { Vec::from_raw_parts(input_buf, input_len, input_len) };
    let mut input = Bin::new(&mut LowMemoryReadableVec::from(input)).unwrap();
    input.1 = BitDepth::from_u8(bit_depth).unwrap();
    input.2 = ColorType::from_u8(color_type).unwrap();

    let output: Vec<u8> = Img::try_from(input).unwrap().into();
    unsafe {
        let result = malloc(size_of::<Vec<u8>>()) as *mut Vec<u8>;
        result.write(output);
        result
    }
}

#[no_mangle]
pub fn img_to_bin(input_buf: *mut u8, input_len: usize) -> *mut Vec<u8> {
    let input = unsafe { Vec::from_raw_parts(input_buf, input_len, input_len) };
    let input = Img::new(input);
    let output = Bin::try_from(input).unwrap();

    let output: Vec<u8> = output.try_into().unwrap();
    unsafe {
        let result = malloc(size_of::<Vec<u8>>()) as *mut Vec<u8>;
        result.write(output);
        result
    }
}

#[no_mangle]
pub fn result_len(src: *mut Vec<u8>) -> usize {
    unsafe { (*src).len() }
}

#[no_mangle]
pub fn result_buf(src: *mut Vec<u8>) -> *mut u8 {
    unsafe { (*src).as_mut_ptr() }
}

#[no_mangle]
pub fn result_free(src: *mut Vec<u8>) {
    unsafe {
        src.drop_in_place();
        free(src as *mut c_void);
    }
}
