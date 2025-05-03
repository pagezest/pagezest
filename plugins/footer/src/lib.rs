use std::ffi::CString;
use std::os::raw::c_char;
use serde_json::{Value, json};

#[unsafe(no_mangle)]
pub extern "C" fn footer(input_ptr: *const c_char, length: usize) -> *const c_char {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr as *const u8, length) };
    let input_string = String::from_utf8_lossy(input_slice).to_string();
    let input_json: Value = serde_json::from_str(&input_string).unwrap();
    let content = input_json.get("content").and_then(|s| s.as_str()).unwrap_or("footer");
    let attribs = input_json.get("attributes").unwrap();
    let color = attribs.get("color").and_then(Value::as_str).unwrap_or("crimson");
    let background = attribs.get("color").and_then(Value::as_str).unwrap_or("beige");
    let style = format!("color: {}; background-color: {}", color, background);
    let footer = format!("<footer style=\"{}\">{}</footer>", style, content);
    let c_string = CString::new(footer).unwrap();
    c_string.into_raw()
}
