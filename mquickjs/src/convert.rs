use std::ffi::{c_char, CString};

use mquickjs_sys::{
    JSValue, JS_EX_NORMAL, JS_GetPropertyStr, JS_GetPropertyUint32, JS_IsNumber,
    JS_NewArray, JS_NewFloat64, JS_NewInt32, JS_NewStringLen, JS_SetPropertyUint32,
    JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_SPECIAL_BITS, JS_ToUint32,
};

use crate::{Context, JsError, Value};

/// Convert Rust values into JavaScript values.
pub trait IntoValue<'ctx> {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError>;
}

/// Convert JavaScript values into Rust values.
pub trait FromValue<'ctx>: Sized {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError>;
}

impl<'ctx> IntoValue<'ctx> for bool {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw = (self as JSValue) << JS_TAG_SPECIAL_BITS | (JS_TAG_BOOL as JSValue);
        Ok(Value::new(ctx.raw_ctx(), raw))
    }
}

impl<'ctx> FromValue<'ctx> for bool {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        value.to_bool()
    }
}

impl<'ctx> IntoValue<'ctx> for i32 {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw = unsafe { JS_NewInt32(ctx.raw_ctx().as_ptr(), self) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to convert i32".to_string(),
            });
        }
        Ok(Value::new(ctx.raw_ctx(), raw))
    }
}

impl<'ctx> FromValue<'ctx> for i32 {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        value.to_i32()
    }
}

impl<'ctx> IntoValue<'ctx> for f64 {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw = unsafe { JS_NewFloat64(ctx.raw_ctx().as_ptr(), self) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to convert f64".to_string(),
            });
        }
        Ok(Value::new(ctx.raw_ctx(), raw))
    }
}

impl<'ctx> FromValue<'ctx> for f64 {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        value.to_f64()
    }
}

impl<'ctx> IntoValue<'ctx> for String {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        self.as_str().into_value(ctx)
    }
}

impl<'ctx> IntoValue<'ctx> for &str {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let bytes = self.as_bytes();
        let raw = unsafe {
            JS_NewStringLen(
                ctx.raw_ctx().as_ptr(),
                bytes.as_ptr() as *const c_char,
                bytes.len(),
            )
        };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to convert string".to_string(),
            });
        }
        Ok(Value::new(ctx.raw_ctx(), raw))
    }
}

impl<'ctx> FromValue<'ctx> for String {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        value.to_string()
    }
}

impl<'ctx, T> IntoValue<'ctx> for Vec<T>
where
    T: IntoValue<'ctx>,
{
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw_ctx = ctx.raw_ctx();
        let raw = unsafe { JS_NewArray(raw_ctx.as_ptr(), self.len() as i32) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to create array".to_string(),
            });
        }

        for (index, item) in self.into_iter().enumerate() {
            let value = item.into_value(ctx)?;
            let result = unsafe {
                JS_SetPropertyUint32(raw_ctx.as_ptr(), raw, index as u32, value.raw())
            };
            if is_exception(result) {
                return Err(JsError::Conversion {
                    message: format!("failed to set array element {index}"),
                });
            }
        }

        Ok(Value::new(raw_ctx, raw))
    }
}

impl<'ctx, T> FromValue<'ctx> for Vec<T>
where
    T: FromValue<'ctx>,
{
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let raw_ctx = value.ctx();
        let length_name = CString::new("length").expect("length contains no nulls");
        let length_raw = unsafe {
            JS_GetPropertyStr(raw_ctx.as_ptr(), value.raw(), length_name.as_ptr())
        };
        if is_exception(length_raw) {
            return Err(JsError::Conversion {
                message: "failed to read array length".to_string(),
            });
        }

        let is_number = unsafe { JS_IsNumber(raw_ctx.as_ptr(), length_raw) };
        if is_number == 0 {
            return Err(JsError::Conversion {
                message: "expected array".to_string(),
            });
        }

        let mut length = 0u32;
        let status = unsafe { JS_ToUint32(raw_ctx.as_ptr(), &mut length, length_raw) };
        if status != 0 {
            return Err(JsError::Conversion {
                message: "failed to convert array length".to_string(),
            });
        }

        let mut out = Vec::with_capacity(length as usize);
        for index in 0..length {
            let elem_raw = unsafe { JS_GetPropertyUint32(raw_ctx.as_ptr(), value.raw(), index) };
            if is_exception(elem_raw) {
                return Err(JsError::Conversion {
                    message: format!("failed to read array element {index}"),
                });
            }
            let elem = Value::new(raw_ctx, elem_raw);
            out.push(T::from_value(elem)?);
        }

        Ok(out)
    }
}

fn is_exception(value: JSValue) -> bool {
    value == js_exception_value()
}

fn js_exception_value() -> JSValue {
    (JS_TAG_EXCEPTION as JSValue) | ((JS_EX_NORMAL as JSValue) << JS_TAG_SPECIAL_BITS)
}
