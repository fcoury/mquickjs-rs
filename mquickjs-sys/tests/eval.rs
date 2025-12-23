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
fn new_context() -> (Vec<usize>, *mut JSContext) {
    let bytes = 8 * 1024 * 1024;
    let words = bytes / std::mem::size_of::<usize>();
    let mut heap = vec![0usize; words];
    let mem_start = heap.as_mut_ptr() as *mut c_void;
    let mem_size = heap.len() * std::mem::size_of::<usize>();
    let ctx = unsafe { JS_NewContext(mem_start, mem_size, &js_stdlib) };
    assert!(!ctx.is_null(), "expected JS_NewContext to return a context");
    (heap, ctx)
}

#[test]
fn eval_arithmetic_expression() {
    let (_heap, ctx) = new_context();
    let filename = CString::new("eval_test").expect("filename");
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
        panic!("expected eval to succeed: {}", error_message(ctx));
    }

    let mut result = 0;
    let status = unsafe { JS_ToInt32(ctx, &mut result, value) };
    assert_eq!(status, 0, "expected JS_ToInt32 to succeed");
    assert_eq!(result, 3, "expected 1 + 2 to equal 3");

    unsafe {
        JS_FreeContext(ctx);
    }
}

#[test]
fn eval_reports_exception_on_invalid_js() {
    let (_heap, ctx) = new_context();
    let filename = CString::new("eval_invalid").expect("filename");
    let script = CString::new("1+").expect("script");

    let value = unsafe {
        JS_Eval(
            ctx,
            script.as_ptr() as *const c_char,
            script.as_bytes().len(),
            filename.as_ptr(),
            JS_EVAL_RETVAL as i32,
        )
    };

    assert_eq!(value, js_exception_value(), "expected eval to fail");

    unsafe {
        JS_FreeContext(ctx);
    }
}
