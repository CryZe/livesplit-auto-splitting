extern crate asl_lang;

use asl_lang::parity_wasm::serialize;
use asl_lang::{Error, Hover, Result, Span};
use std::ffi::CStr;
use std::fmt::Write;
use std::os::raw::c_char;

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
pub mod wasm {
    use std::alloc::{alloc as allocate, dealloc as deallocate, Layout};

    #[no_mangle]
    pub unsafe extern "C" fn alloc(size: usize) -> *mut u8 {
        allocate(Layout::from_size_align_unchecked(size, 1))
    }

    #[no_mangle]
    pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
        deallocate(ptr, Layout::from_size_align_unchecked(size, 1))
    }
}

unsafe fn str(s: *const c_char) -> &'static str {
    if s.is_null() {
        ""
    } else {
        CStr::from_ptr(s as _).to_str().unwrap()
    }
}

pub type FFIResult<T> = Result<Option<Box<T>>>;

#[no_mangle]
pub unsafe extern "C" fn ASL_compile(text: *const c_char) -> Box<FFIResult<Vec<u8>>> {
    Box::new(asl_lang::compile(str(text)).map(|m| Some(Box::new(serialize(m).unwrap()))))
}

#[no_mangle]
pub unsafe extern "C" fn ASL_hover(
    text: *const c_char,
    line: usize,
    column: usize,
) -> Box<FFIResult<Hover>> {
    Box::new(asl_lang::hover(str(text), line, column).map(|h| h.map(Box::new)))
}

#[no_mangle]
pub unsafe extern "C" fn ASL_go_to_definition(
    text: *const c_char,
    line: usize,
    column: usize,
) -> Box<FFIResult<Span>> {
    Box::new(asl_lang::go_to_definition(str(text), line, column).map(|h| h.map(Box::new)))
}

#[no_mangle]
pub unsafe extern "C" fn ASL_find_all_references(
    text: *const c_char,
    line: usize,
    column: usize,
) -> Box<FFIResult<Vec<Span>>> {
    Box::new(asl_lang::find_all_references(str(text), line, column).map(|h| h.map(Box::new)))
}

#[no_mangle]
pub extern "C" fn Result_is_ok(this: &FFIResult<usize>) -> bool {
    this.is_ok()
}

#[no_mangle]
pub extern "C" fn Result_error(this: Box<FFIResult<usize>>) -> Box<Error> {
    Box::new(this.err().unwrap())
}

#[no_mangle]
pub extern "C" fn Result_ok(this: Box<FFIResult<usize>>) -> Option<Box<usize>> {
    this.ok().unwrap()
}

#[no_mangle]
pub extern "C" fn Error_msg_ptr(this: &Error) -> *const u8 {
    this.message.as_ptr()
}

#[no_mangle]
pub extern "C" fn Error_msg_len(this: &Error) -> usize {
    this.message.len()
}

#[no_mangle]
pub extern "C" fn Error_span(this: &Error) -> Option<&Span> {
    this.span.as_ref()
}

#[no_mangle]
pub extern "C" fn Hover_ty(this: &Hover) -> Box<Vec<u8>> {
    let text = if let Some(params) = &this.params {
        let mut text = String::from("fn (");
        for (i, param) in params.into_iter().enumerate() {
            if i != 0 {
                text.push_str(", ");
            }
            write!(text, "{}", param);
        }
        write!(text, ") -> {}", this.ty);
        text
    } else {
        format!("{}", this.ty)
    };
    Box::new(text.into_bytes())
}

#[no_mangle]
pub extern "C" fn Hover_span(this: &Hover) -> &Span {
    &this.span
}

#[no_mangle]
pub extern "C" fn Span_line_from(this: &Span) -> usize {
    this.from.0
}

#[no_mangle]
pub extern "C" fn Span_column_from(this: &Span) -> usize {
    this.from.1
}

#[no_mangle]
pub extern "C" fn Span_line_to(this: &Span) -> usize {
    this.to.0
}

#[no_mangle]
pub extern "C" fn Span_column_to(this: &Span) -> usize {
    this.to.1
}

#[no_mangle]
pub extern "C" fn Spans_get(this: &Vec<Span>, index: usize) -> Option<&Span> {
    this.get(index)
}

#[no_mangle]
pub extern "C" fn Buf_len(this: &Vec<u8>) -> usize {
    this.len()
}

#[no_mangle]
pub extern "C" fn Buf_as_ptr(this: &Vec<u8>) -> *const u8 {
    this.as_ptr()
}

#[no_mangle]
pub extern "C" fn Buf_drop(this: Box<Vec<u8>>) {
    drop(this)
}
