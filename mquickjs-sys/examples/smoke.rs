use std::ffi::{c_char, c_void, CString};

use mquickjs_sys::{
    js_stdlib, JSContext, JS_EVAL_RETVAL, JS_Eval, JS_EX_NORMAL, JS_FreeContext, JS_GetErrorStr,
    JS_NewContext, JS_TAG_EXCEPTION, JS_TAG_SPECIAL_BITS, JS_ToInt32, JSValue,
};

fn js_exception_value() -> JSValue {
    (JS_TAG_EXCEPTION as JSValue) | ((JS_EX_NORMAL as JSValue) << JS_TAG_SPECIAL_BITS)
}

fn error_message(ctx: *mut JSContext) -> String {
    let mut buf = [0u8; 256];
    unsafe {
        JS_GetErrorStr(ctx, buf.as_mut_ptr() as *mut c_char, buf.len());
    }
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

fn main() {
    // mquickjs expects a preallocated, aligned memory buffer (>= 1024 bytes).
    let bytes = 1024 * 1024;
    let words = bytes / std::mem::size_of::<usize>();
    let mut heap = vec![0usize; words];
    let mem_start = heap.as_mut_ptr() as *mut c_void;
    let mem_size = heap.len() * std::mem::size_of::<usize>();

    let ctx = unsafe { JS_NewContext(mem_start, mem_size, &js_stdlib) };
    if ctx.is_null() {
        eprintln!("failed to create JSContext");
        return;
    }

    let filename = CString::new("smoke").expect("filename");
    let script = CString::new("1+2").expect("script");

    let value = unsafe {
        JS_Eval(
            ctx,
            script.as_ptr() as *const c_char,
            script.as_bytes().len(),
            filename.as_ptr(),
            JS_EVAL_RETVAL as i32,
        )
    };

    if value == js_exception_value() {
        eprintln!("eval failed: {}", error_message(ctx));
    } else {
        let mut result = 0;
        let status = unsafe { JS_ToInt32(ctx, &mut result, value) };
        if status == 0 {
            println!("result: {result}");
        } else {
            eprintln!("failed to convert result to int");
        }
    }

    unsafe {
        JS_FreeContext(ctx);
    }
}
