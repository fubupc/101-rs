use std::{
    ffi::c_uchar,
    ffi::{c_uint, c_ulong},
};

extern "C" {
    fn CRC32(data: *const c_uchar, data_length: c_ulong) -> c_uint;
}

fn crc32(data: &[u8]) -> u32 {
    unsafe { CRC32(data.as_ptr(), data.len() as u64) }
}

fn main() {
    println!("{:#x}", crc32(b"12345678"));
}
