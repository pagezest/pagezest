use std::ffi::CString;
use std::os::raw::c_char;

use post_flatbuffers::pagezest_markdown::{root_as_document, TokenType};
mod post_flatbuffers;

#[unsafe(no_mangle)]
pub extern "C" fn toc(input_ptr: *const c_char, length: usize) -> *const c_char {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr as *const u8, length) };
    let doc = root_as_document(&input_slice).unwrap();
    let mut toc_items: Vec<String> = Vec::new();
    if let Some(tokens) = doc.tokens() {
        for node in tokens {
            if node.type_() == TokenType::HEADING {
                let heading = node.value_as_heading().unwrap();
                if heading.depth() == 1 {
                    toc_items.push(format!("<li>{}</li>", heading.text()));
                }
            }
        }
    }
    let output = format!("<h1>Table of content</h1><ul>{}</ul>", toc_items.join("\n"));
    let c_string = CString::new(output).unwrap();
    c_string.into_raw()
}
