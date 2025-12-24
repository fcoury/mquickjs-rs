use mquickjs_sys::{
    JSValue, JS_Call, JS_EX_NORMAL, JS_IsFunction, JS_PushArg, JS_StackCheck,
    JS_TAG_EXCEPTION, JS_TAG_NULL, JS_TAG_SPECIAL_BITS,
};

use crate::context::exception_error;
use crate::{Context, IntoValue, JsError, Value};

/// Wrapper around a JavaScript function value.
#[derive(Debug, Clone, Copy)]
pub struct Function<'ctx> {
    ctx: &'ctx Context,
    value: Value<'ctx>,
}

impl<'ctx> Function<'ctx> {
    /// Create a function wrapper from a value.
    pub fn from_value(ctx: &'ctx Context, value: Value<'ctx>) -> Result<Self, JsError> {
        ensure_same_context(ctx, value)?;
        let is_function = unsafe { JS_IsFunction(ctx.raw_ctx().as_ptr(), value.raw()) };
        if is_function == 0 {
            return Err(JsError::Conversion {
                message: "expected function".to_string(),
            });
        }
        Ok(Self { ctx, value })
    }

    /// Call the function with the provided arguments.
    pub fn call(&self, args: &[Value<'ctx>]) -> Result<Value<'ctx>, JsError> {
        for arg in args {
            ensure_same_context(self.ctx, *arg)?;
        }

        if unsafe { JS_StackCheck(self.ctx.raw_ctx().as_ptr(), (args.len() + 2) as u32) } != 0 {
            return Err(JsError::Runtime {
                message: "stack overflow when calling function".to_string(),
            });
        }

        for arg in args.iter().rev() {
            unsafe {
                JS_PushArg(self.ctx.raw_ctx().as_ptr(), arg.raw());
            }
        }

        unsafe {
            JS_PushArg(self.ctx.raw_ctx().as_ptr(), self.value.raw());
            JS_PushArg(self.ctx.raw_ctx().as_ptr(), js_null_value());
        }

        let result = unsafe { JS_Call(self.ctx.raw_ctx().as_ptr(), args.len() as i32) };
        if is_exception(result) {
            return Err(exception_error(self.ctx.raw_ctx().as_ptr()));
        }

        Ok(Value::new(self.ctx.raw_ctx(), result))
    }

    /// Call the function with no arguments.
    pub fn call0(&self) -> Result<Value<'ctx>, JsError> {
        self.call(&[])
    }

    /// Call the function with a single argument.
    pub fn call1<T: IntoValue<'ctx>>(&self, arg: T) -> Result<Value<'ctx>, JsError> {
        let value = arg.into_value(self.ctx)?;
        self.call(&[value])
    }
}

fn ensure_same_context(ctx: &Context, value: Value<'_>) -> Result<(), JsError> {
    if ctx.raw_ctx() != value.ctx() {
        return Err(JsError::Conversion {
            message: "value does not belong to context".to_string(),
        });
    }
    Ok(())
}

fn is_exception(value: JSValue) -> bool {
    value == js_exception_value()
}

fn js_exception_value() -> JSValue {
    (JS_TAG_EXCEPTION as JSValue) | ((JS_EX_NORMAL as JSValue) << JS_TAG_SPECIAL_BITS)
}

fn js_null_value() -> JSValue {
    JS_TAG_NULL as JSValue
}
