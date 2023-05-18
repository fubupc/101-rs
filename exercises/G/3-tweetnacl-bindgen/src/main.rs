use std::mem::MaybeUninit;

pub fn crypto_hash_sha512_tweet(data: &[u8]) -> [u8; 64] {
    let mut out: MaybeUninit<[u8; 64]> = MaybeUninit::uninit();
    unsafe {
        tweetnacl_bindgen::bindings::crypto_hashblocks_sha512_tweet(
            out.as_mut_ptr() as *mut _,
            data.as_ptr(),
            data.len() as _,
        );

        out.assume_init()
    }
}

fn main() {
    println!("Hello, world!");
}
