use std::ffi::CString;

use mquickjs_sys::{
    JSValue, JS_EX_NORMAL, JS_GetPropertyStr, JS_GetPropertyUint32, JS_IsNumber,
    JS_SetPropertyStr, JS_SetPropertyUint32, JS_TAG_EXCEPTION, JS_TAG_SPECIAL_BITS,
    JS_ToUint32,
};

use crate::{Context, FromValue, IntoValue, JsError, Value};

/// Wrapper around a JavaScript object value.
#[derive(Debug, Clone, Copy)]
pub struct Object<'ctx> {
    ctx: &'ctx Context,
    value: Value<'ctx>,
}

impl<'ctx> Object<'ctx> {
    /// Create an object wrapper from a value.
    pub fn from_value(ctx: &'ctx Context, value: Value<'ctx>) -> Result<Self, JsError> {
        ensure_same_context(ctx, value)?;
        Ok(Self { ctx, value })
    }

    /// Get a property and convert it into a Rust value.
    pub fn get<T: FromValue<'ctx>>(&self, name: &str) -> Result<T, JsError> {
        let name_str = name.to_string();
        let name = CString::new(name).map_err(|_| JsError::Conversion {
            message: "property name contains null byte".to_string(),
        })?;

        let raw = unsafe { JS_GetPropertyStr(self.ctx.raw_ctx().as_ptr(), self.value.raw(), name.as_ptr()) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: format!("failed to get property '{name_str}'"),
            });
        }

        let value = Value::new(self.ctx.raw_ctx(), raw);
        T::from_value(value)
    }

    /// Set a property from a Rust value.
    pub fn set<T: IntoValue<'ctx>>(&self, name: &str, value: T) -> Result<(), JsError> {
        let name_str = name.to_string();
        let name = CString::new(name).map_err(|_| JsError::Conversion {
            message: "property name contains null byte".to_string(),
        })?;
        let value = value.into_value(self.ctx)?;
        let result = unsafe {
            JS_SetPropertyStr(
                self.ctx.raw_ctx().as_ptr(),
                self.value.raw(),
                name.as_ptr(),
                value.raw(),
            )
        };
        if is_exception(result) {
            return Err(JsError::Conversion {
                message: format!("failed to set property '{name_str}'"),
            });
        }
        Ok(())
    }
}

/// Wrapper around a JavaScript array value.
#[derive(Debug, Clone, Copy)]
pub struct Array<'ctx> {
    ctx: &'ctx Context,
    value: Value<'ctx>,
}

impl<'ctx> Array<'ctx> {
    /// Create an array wrapper from a value.
    pub fn from_value(ctx: &'ctx Context, value: Value<'ctx>) -> Result<Self, JsError> {
        ensure_same_context(ctx, value)?;
        let array = Self { ctx, value };
        array.length()?;
        Ok(array)
    }

    /// Push a value onto the array.
    pub fn push<T: IntoValue<'ctx>>(&self, value: T) -> Result<(), JsError> {
        let index = self.length()?;
        let value = value.into_value(self.ctx)?;
        let result = unsafe {
            JS_SetPropertyUint32(
                self.ctx.raw_ctx().as_ptr(),
                self.value.raw(),
                index,
                value.raw(),
            )
        };
        if is_exception(result) {
            return Err(JsError::Conversion {
                message: format!("failed to set array element {index}"),
            });
        }
        Ok(())
    }

    /// Get an element from the array.
    pub fn get<T: FromValue<'ctx>>(&self, index: usize) -> Result<T, JsError> {
        let raw = unsafe {
            JS_GetPropertyUint32(self.ctx.raw_ctx().as_ptr(), self.value.raw(), index as u32)
        };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: format!("failed to get array element {index}"),
            });
        }
        let value = Value::new(self.ctx.raw_ctx(), raw);
        T::from_value(value)
    }

    fn length(&self) -> Result<u32, JsError> {
        let name = CString::new("length").expect("length contains no null bytes");
        let raw = unsafe {
            JS_GetPropertyStr(self.ctx.raw_ctx().as_ptr(), self.value.raw(), name.as_ptr())
        };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to read array length".to_string(),
            });
        }

        let is_number = unsafe { JS_IsNumber(self.ctx.raw_ctx().as_ptr(), raw) };
        if is_number == 0 {
            return Err(JsError::Conversion {
                message: "expected array".to_string(),
            });
        }

        let mut length = 0u32;
        let status = unsafe { JS_ToUint32(self.ctx.raw_ctx().as_ptr(), &mut length, raw) };
        if status != 0 {
            return Err(JsError::Conversion {
                message: "failed to convert array length".to_string(),
            });
        }

        Ok(length)
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
