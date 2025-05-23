use std::ffi::CString;
use std::os::raw::c_char;

use post_flatbuffers::pagezest_markdown::{root_as_document, TokenType};
mod post_flatbuffers;
use std::time::{SystemTime, UNIX_EPOCH};

static mut SEED: u32 = 0x1234ABCD;

fn simple_random_u32() -> u32 {
    unsafe {
        // Constants from Numerical Recipes LCG
        SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
        SEED
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn time(input_ptr: *const c_char, length: usize) -> *const c_char {
    //let now = SystemTime::now()
    //    .duration_since(UNIX_EPOCH)
    //    .expect("Time went backwards");

    //let seconds = now.as_secs();
    //let nanos = now.subsec_nanos();

    let n = simple_random_u32();
    let output = format!("random value {n}");
    let c_string = CString::new(output).unwrap();
    c_string.into_raw()
}
