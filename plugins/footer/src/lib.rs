use std::ffi::CString;
use std::os::raw::c_char;
use serde_json::Value;

#[unsafe(no_mangle)]
pub extern "C" fn footer(input_ptr: *const c_char, length: usize) -> *const c_char {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr as *const u8, length) };
    let input_string = String::from_utf8_lossy(input_slice).to_string();
    let input_json: Value = serde_json::from_str(&input_string).unwrap();
    let tag = input_json.get("tag").and_then(|s| s.as_str()).unwrap();
    let c_string = CString::new(tag).unwrap();
    c_string.into_raw()
}
