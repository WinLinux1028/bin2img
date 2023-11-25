use std::{ffi::c_void, mem::size_of};

use bin_img_conv::{Bin, BitDepth, ColorType, Img};

extern "C" {
    fn malloc(_: usize) -> *mut c_void;
    fn free(_: *mut c_void);
}

#[no_mangle]
pub fn bin_to_img(input: *mut Vec<u8>, bit_depth: u8, color_type: u8) -> *mut Vec<u8> {
    let mut input = unsafe { Bin::new(&mut (*input).as_slice()).unwrap() };
    input.1 = BitDepth::from_u8(bit_depth).unwrap();
    input.2 = ColorType::from_u8(color_type).unwrap();

    let result = Img::try_from(input).unwrap().into();
    unsafe {
        let output = buf_alloc();
        output.write(result);

        output
    }
}

#[no_mangle]
pub fn img_to_bin(input: *mut Vec<u8>) -> *mut Vec<u8> {
    let input = unsafe { Img::new((*input).as_slice()) };

    let result = Bin::try_from(input).unwrap().try_into().unwrap();
    unsafe {
        let output = buf_alloc();
        output.write(result);

        output
    }
}

#[no_mangle]
pub fn buf_alloc() -> *mut Vec<u8> {
    unsafe {
        let buf = malloc(size_of::<Vec<u8>>()) as *mut Vec<u8>;
        buf.write(Vec::with_capacity(1));
        buf
    }
}

#[no_mangle]
pub fn buf_resize(src: *mut Vec<u8>, new_len: usize) {
    unsafe {
        (*src).resize(new_len, 0);
        (*src).shrink_to(1);
    }
}

#[no_mangle]
pub fn buf_inner(src: *mut Vec<u8>) -> *mut u8 {
    unsafe { (*src).as_mut_ptr() }
}

#[no_mangle]
pub fn buf_len(src: *mut Vec<u8>) -> usize {
    unsafe { (*src).len() }
}

#[no_mangle]
pub fn buf_free(src: *mut Vec<u8>) {
    unsafe {
        src.drop_in_place();
        free(src as *mut c_void);
    }
}
