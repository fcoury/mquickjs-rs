use std::ffi::{c_char, c_int};

use mquickjs_sys::{
    JSContext, JSValue, JS_Eval, JS_FreeContext, JS_NewContext, JS_NewContext2,
    JSSTDLibraryDef,
};

#[test]
fn bindings_expose_core_types() {
    let _ctx: *mut JSContext = std::ptr::null_mut();
    let _value: JSValue = 0;
    let _stdlib: *const JSSTDLibraryDef = std::ptr::null();
    let _ = (_ctx, _value, _stdlib);
}

#[test]
fn bindings_expose_core_functions() {
    let _new_ctx: unsafe extern "C" fn(*mut std::ffi::c_void, usize, *const JSSTDLibraryDef) -> *mut JSContext =
        JS_NewContext;
    let _new_ctx2: unsafe extern "C" fn(
        *mut std::ffi::c_void,
        usize,
        *const JSSTDLibraryDef,
        c_int,
    ) -> *mut JSContext = JS_NewContext2;
    let _eval: unsafe extern "C" fn(
        *mut JSContext,
        *const c_char,
        usize,
        *const c_char,
        c_int,
    ) -> JSValue = JS_Eval;
    let _free: unsafe extern "C" fn(*mut JSContext) = JS_FreeContext;

    let _ = (_new_ctx, _new_ctx2, _eval, _free);
}
