use std::ffi::c_void;

use mquickjs_sys::{js_stdlib, JSContext, JS_FreeContext, JS_NewContext};

#[test]
fn context_lifecycle_with_buffer() {
    let words = (1024 * 1024) / std::mem::size_of::<usize>();
    let mut heap = vec![0usize; words];
    let mem_start = heap.as_mut_ptr() as *mut c_void;
    let mem_size = heap.len() * std::mem::size_of::<usize>();

    let ctx: *mut JSContext = unsafe { JS_NewContext(mem_start, mem_size, &js_stdlib) };
    assert!(!ctx.is_null(), "expected JS_NewContext to return a context");

    unsafe {
        JS_FreeContext(ctx);
    }
}
