//! Function binding utilities.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_int;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::NonNull;
use std::sync::Once;

use mquickjs_sys::{JSContext, JSValue, JS_NewString, JS_SetHostCallback, JS_Throw, JS_ToInt32};

use crate::error::JsError;
use crate::value::Value;

type Callback = dyn for<'ctx> Fn(&[Value<'ctx>]) -> Result<Value<'ctx>, JsError>;

struct Registry {
    next_id: u32,
    callbacks: HashMap<u32, Box<Callback>>,
}

thread_local! {
    static REGISTRY: RefCell<HashMap<*mut JSContext, Registry>> =
        RefCell::new(HashMap::new());
}

static HOST_CALLBACK_INIT: Once = Once::new();

pub(crate) fn register_context(ctx: NonNull<JSContext>) {
    ensure_host_callback();
    REGISTRY.with(|registry| {
        registry
            .borrow_mut()
            .entry(ctx.as_ptr())
            .or_insert_with(|| Registry {
                next_id: 1,
                callbacks: HashMap::new(),
            });
    });
}

pub(crate) fn unregister_context(ctx: *mut JSContext) {
    REGISTRY.with(|registry| {
        registry.borrow_mut().remove(&ctx);
    });
}

pub(crate) fn register_callback(
    ctx: NonNull<JSContext>,
    callback: Box<Callback>,
) -> u32 {
    register_context(ctx);
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        let entry = registry
            .get_mut(&ctx.as_ptr())
            .expect("context registry missing");
        let id = entry.next_id;
        entry.next_id = entry.next_id.wrapping_add(1);
        entry.callbacks.insert(id, callback);
        id
    })
}

fn ensure_host_callback() {
    HOST_CALLBACK_INIT.call_once(|| unsafe {
        JS_SetHostCallback(Some(host_callback));
    });
}

unsafe extern "C" fn host_callback(
    ctx_ptr: *mut JSContext,
    _this_val: *mut JSValue,
    argc: c_int,
    argv: *mut JSValue,
) -> JSValue {
    if ctx_ptr.is_null() || argv.is_null() || argc <= 0 {
        return throw_string(ctx_ptr, "missing callback id");
    }

    let args = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
    let mut id = 0i32;
    if unsafe { JS_ToInt32(ctx_ptr, &mut id, args[0]) } != 0 {
        return throw_string(ctx_ptr, "invalid callback id");
    }

    let ctx = match NonNull::new(ctx_ptr) {
        Some(ctx) => ctx,
        None => return throw_string(ctx_ptr, "invalid context"),
    };

    let values: Vec<Value> = args[1..]
        .iter()
        .map(|raw| Value::new(ctx, *raw))
        .collect();

    let callback = REGISTRY.with(|registry| {
        let registry = registry.borrow();
        let entry = registry.get(&ctx.as_ptr())?;
        let cb = entry.callbacks.get(&(id as u32))?;
        Some(cb.as_ref() as *const Callback)
    });

    let Some(callback) = callback else {
        return throw_string(ctx.as_ptr(), "unknown callback id");
    };

    let outcome = catch_unwind(AssertUnwindSafe(|| unsafe { (&*callback)(&values) }));
    match outcome {
        Ok(Ok(value)) => value.raw(),
        Ok(Err(err)) => throw_string(ctx.as_ptr(), &err.to_string()),
        Err(_) => throw_string(ctx.as_ptr(), "callback panicked"),
    }
}

fn throw_string(ctx: *mut JSContext, message: &str) -> JSValue {
    let message = CString::new(message).unwrap_or_else(|_| CString::new("error").unwrap());
    unsafe { JS_Throw(ctx, JS_NewString(ctx, message.as_ptr())) }
}
