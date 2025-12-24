use std::collections::{HashMap, VecDeque};
use std::ffi::{c_char, CString};
use std::ptr::NonNull;

use mquickjs_sys::{
    JSCStringBuf, JSContext, JSValue, JS_Call, JS_EX_NORMAL, JS_GetGlobalObject,
    JS_GetPropertyStr, JS_GetPropertyUint32, JS_IsNumber, JS_NewArray,
    JS_NewFloat64, JS_NewInt32, JS_NewInt64, JS_NewObject, JS_NewStringLen,
    JS_NewUint32, JS_PushArg, JS_SetPropertyStr, JS_SetPropertyUint32,
    JS_StackCheck, JS_TAG_BOOL, JS_TAG_EXCEPTION, JS_TAG_NULL, JS_TAG_SPECIAL_BITS,
    JS_TAG_UNDEFINED, JS_ToCStringLen, JS_ToNumber, JS_ToString, JS_ToUint32,
};

use crate::{Context, JsError, Value};

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// Convert Rust values into JavaScript values.
pub trait IntoValue<'ctx> {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError>;
}

/// Convert JavaScript values into Rust values.
pub trait FromValue<'ctx>: Sized {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError>;
}

/// Wrapper for JS-style coercions when converting from values.
#[derive(Debug, Clone, PartialEq)]
pub struct Coerced<T>(pub T);

impl<T> Coerced<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
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

impl<'ctx> IntoValue<'ctx> for i64 {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw = unsafe { JS_NewInt64(ctx.raw_ctx().as_ptr(), self) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to convert i64".to_string(),
            });
        }
        Ok(Value::new(ctx.raw_ctx(), raw))
    }
}

impl<'ctx> FromValue<'ctx> for i64 {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = value.to_f64()?;
        let number = ensure_integer(number, "i64")?;
        if number.abs() > MAX_SAFE_INTEGER {
            return Err(JsError::Conversion {
                message: "i64 out of safe JS integer range".to_string(),
            });
        }
        Ok(number as i64)
    }
}

impl<'ctx> IntoValue<'ctx> for u64 {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw = if self <= u32::MAX as u64 {
            unsafe { JS_NewUint32(ctx.raw_ctx().as_ptr(), self as u32) }
        } else if self as f64 <= MAX_SAFE_INTEGER {
            unsafe { JS_NewFloat64(ctx.raw_ctx().as_ptr(), self as f64) }
        } else {
            return Err(JsError::Conversion {
                message: "u64 out of safe JS integer range".to_string(),
            });
        };

        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to convert u64".to_string(),
            });
        }
        Ok(Value::new(ctx.raw_ctx(), raw))
    }
}

impl<'ctx> FromValue<'ctx> for u64 {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = value.to_f64()?;
        let number = ensure_integer(number, "u64")?;
        if number < 0.0 || number > MAX_SAFE_INTEGER {
            return Err(JsError::Conversion {
                message: "u64 out of safe JS integer range".to_string(),
            });
        }
        Ok(number as u64)
    }
}

impl<'ctx> IntoValue<'ctx> for usize {
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        (self as u64).into_value(ctx)
    }
}

impl<'ctx> FromValue<'ctx> for usize {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = u64::from_value(value)?;
        usize::try_from(number).map_err(|_| JsError::Conversion {
            message: "usize out of range".to_string(),
        })
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

impl<'ctx> FromValue<'ctx> for Coerced<i32> {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = coerce_number(value)?;
        let number = ensure_integer(number, "i32")?;
        if number < i32::MIN as f64 || number > i32::MAX as f64 {
            return Err(JsError::Conversion {
                message: "i32 out of range".to_string(),
            });
        }
        Ok(Coerced(number as i32))
    }
}

impl<'ctx> FromValue<'ctx> for Coerced<i64> {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = coerce_number(value)?;
        let number = ensure_integer(number, "i64")?;
        if number.abs() > MAX_SAFE_INTEGER {
            return Err(JsError::Conversion {
                message: "i64 out of safe JS integer range".to_string(),
            });
        }
        Ok(Coerced(number as i64))
    }
}

impl<'ctx> FromValue<'ctx> for Coerced<u64> {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = coerce_number(value)?;
        let number = ensure_integer(number, "u64")?;
        if number < 0.0 || number > MAX_SAFE_INTEGER {
            return Err(JsError::Conversion {
                message: "u64 out of safe JS integer range".to_string(),
            });
        }
        Ok(Coerced(number as u64))
    }
}

impl<'ctx> FromValue<'ctx> for Coerced<usize> {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = Coerced::<u64>::from_value(value)?.0;
        let converted = usize::try_from(number).map_err(|_| JsError::Conversion {
            message: "usize out of range".to_string(),
        })?;
        Ok(Coerced(converted))
    }
}

impl<'ctx> FromValue<'ctx> for Coerced<f64> {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let number = coerce_number(value)?;
        Ok(Coerced(number))
    }
}

impl<'ctx> FromValue<'ctx> for Coerced<String> {
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let ctx = value.ctx();
        let string_value = unsafe { JS_ToString(ctx.as_ptr(), value.raw()) };
        if is_exception(string_value) {
            return Err(JsError::Conversion {
                message: "failed to coerce string".to_string(),
            });
        }
        Ok(Coerced(string_from_js(ctx.as_ptr(), string_value)?))
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

impl<'ctx, T> IntoValue<'ctx> for Option<T>
where
    T: IntoValue<'ctx>,
{
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        match self {
            Some(value) => value.into_value(ctx),
            None => Ok(Value::new(ctx.raw_ctx(), js_null_value())),
        }
    }
}

impl<'ctx, T> FromValue<'ctx> for Option<T>
where
    T: FromValue<'ctx>,
{
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        if is_null_or_undefined(value.raw()) {
            return Ok(None);
        }
        Ok(Some(T::from_value(value)?))
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
        let length = array_length(raw_ctx, value.raw())?;

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

impl<'ctx, T> IntoValue<'ctx> for VecDeque<T>
where
    T: IntoValue<'ctx>,
{
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let vec: Vec<T> = self.into_iter().collect();
        vec.into_value(ctx)
    }
}

impl<'ctx, T> FromValue<'ctx> for VecDeque<T>
where
    T: FromValue<'ctx>,
{
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let vec = Vec::<T>::from_value(value)?;
        Ok(vec.into_iter().collect())
    }
}

impl<'ctx, T> IntoValue<'ctx> for HashMap<String, T>
where
    T: IntoValue<'ctx>,
{
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw_ctx = ctx.raw_ctx();
        let raw = unsafe { JS_NewObject(raw_ctx.as_ptr()) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to create object".to_string(),
            });
        }

        for (key, value) in self {
            let name = CString::new(key).map_err(|_| JsError::Conversion {
                message: "object key contains null byte".to_string(),
            })?;
            let value = value.into_value(ctx)?;
            let result = unsafe {
                JS_SetPropertyStr(raw_ctx.as_ptr(), raw, name.as_ptr(), value.raw())
            };
            if is_exception(result) {
                return Err(JsError::Conversion {
                    message: "failed to set object property".to_string(),
                });
            }
        }

        Ok(Value::new(raw_ctx, raw))
    }
}

impl<'ctx, T> FromValue<'ctx> for HashMap<String, T>
where
    T: FromValue<'ctx>,
{
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let raw_ctx = value.ctx();
        let keys = object_keys(raw_ctx, value)?;
        let mut out = HashMap::with_capacity(keys.len());

        for key in keys {
            let name = CString::new(key.clone()).map_err(|_| JsError::Conversion {
                message: "object key contains null byte".to_string(),
            })?;
            let raw = unsafe {
                JS_GetPropertyStr(raw_ctx.as_ptr(), value.raw(), name.as_ptr())
            };
            if is_exception(raw) {
                return Err(JsError::Conversion {
                    message: format!("failed to read property '{key}'"),
                });
            }
            let item = Value::new(raw_ctx, raw);
            let converted = T::from_value(item)?;
            out.insert(key, converted);
        }

        Ok(out)
    }
}

impl<'ctx, A, B> IntoValue<'ctx> for (A, B)
where
    A: IntoValue<'ctx>,
    B: IntoValue<'ctx>,
{
    fn into_value(self, ctx: &'ctx Context) -> Result<Value<'ctx>, JsError> {
        let raw_ctx = ctx.raw_ctx();
        let raw = unsafe { JS_NewArray(raw_ctx.as_ptr(), 2) };
        if is_exception(raw) {
            return Err(JsError::Conversion {
                message: "failed to create tuple array".to_string(),
            });
        }

        let first = self.0.into_value(ctx)?;
        let second = self.1.into_value(ctx)?;

        let result = unsafe { JS_SetPropertyUint32(raw_ctx.as_ptr(), raw, 0, first.raw()) };
        if is_exception(result) {
            return Err(JsError::Conversion {
                message: "failed to set tuple element 0".to_string(),
            });
        }

        let result = unsafe { JS_SetPropertyUint32(raw_ctx.as_ptr(), raw, 1, second.raw()) };
        if is_exception(result) {
            return Err(JsError::Conversion {
                message: "failed to set tuple element 1".to_string(),
            });
        }

        Ok(Value::new(raw_ctx, raw))
    }
}

impl<'ctx, A, B> FromValue<'ctx> for (A, B)
where
    A: FromValue<'ctx>,
    B: FromValue<'ctx>,
{
    fn from_value(value: Value<'ctx>) -> Result<Self, JsError> {
        let raw_ctx = value.ctx();
        let length = array_length(raw_ctx, value.raw())?;
        if length != 2 {
            return Err(JsError::Conversion {
                message: "expected array of length 2".to_string(),
            });
        }

        let first_raw = unsafe { JS_GetPropertyUint32(raw_ctx.as_ptr(), value.raw(), 0) };
        if is_exception(first_raw) {
            return Err(JsError::Conversion {
                message: "failed to read tuple element 0".to_string(),
            });
        }
        let second_raw = unsafe { JS_GetPropertyUint32(raw_ctx.as_ptr(), value.raw(), 1) };
        if is_exception(second_raw) {
            return Err(JsError::Conversion {
                message: "failed to read tuple element 1".to_string(),
            });
        }

        let first = A::from_value(Value::new(raw_ctx, first_raw))?;
        let second = B::from_value(Value::new(raw_ctx, second_raw))?;
        Ok((first, second))
    }
}

fn is_exception(value: JSValue) -> bool {
    value == js_exception_value()
}

fn js_exception_value() -> JSValue {
    (JS_TAG_EXCEPTION as JSValue) | ((JS_EX_NORMAL as JSValue) << JS_TAG_SPECIAL_BITS)
}

fn js_special_value(tag: u32, payload: u64) -> JSValue {
    ((payload << JS_TAG_SPECIAL_BITS) as JSValue) | (tag as JSValue)
}

fn js_null_value() -> JSValue {
    js_special_value(JS_TAG_NULL as u32, 0)
}

fn is_null_or_undefined(value: JSValue) -> bool {
    let tag = value_tag(value);
    tag == JS_TAG_NULL as u32 || tag == JS_TAG_UNDEFINED as u32
}

fn value_tag(value: JSValue) -> u32 {
    let mask = (1u64 << JS_TAG_SPECIAL_BITS) - 1;
    (value as u64 & mask) as u32
}

fn array_length(raw_ctx: NonNull<JSContext>, value: JSValue) -> Result<u32, JsError> {
    let length_name = CString::new("length").expect("length contains no nulls");
    let length_raw = unsafe { JS_GetPropertyStr(raw_ctx.as_ptr(), value, length_name.as_ptr()) };
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

    Ok(length)
}

fn ensure_integer(value: f64, name: &str) -> Result<f64, JsError> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(JsError::Conversion {
            message: format!("{name} expected integer"),
        });
    }
    Ok(value)
}

fn coerce_number(value: Value<'_>) -> Result<f64, JsError> {
    let raw_ctx = value.ctx();
    let mut out = 0f64;
    let status = unsafe { JS_ToNumber(raw_ctx.as_ptr(), &mut out, value.raw()) };
    if status != 0 {
        return Err(JsError::Conversion {
            message: "failed to coerce number".to_string(),
        });
    }
    Ok(out)
}

fn string_from_js(ctx: *mut JSContext, value: JSValue) -> Result<String, JsError> {
    let mut buf = JSCStringBuf { buf: [0u8; 5] };
    let mut len = 0usize;
    let ptr = unsafe { JS_ToCStringLen(ctx, &mut len, value, &mut buf) };
    if ptr.is_null() {
        return Err(JsError::Conversion {
            message: "failed to convert to string".to_string(),
        });
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn object_keys(raw_ctx: NonNull<JSContext>, value: Value<'_>) -> Result<Vec<String>, JsError> {
    if unsafe { JS_StackCheck(raw_ctx.as_ptr(), 3) } != 0 {
        return Err(JsError::Conversion {
            message: "stack overflow when reading object keys".to_string(),
        });
    }

    let global = unsafe { JS_GetGlobalObject(raw_ctx.as_ptr()) };
    if is_exception(global) {
        return Err(JsError::Conversion {
            message: "failed to read global object".to_string(),
        });
    }

    let object_name = CString::new("Object").expect("Object contains no nulls");
    let keys_name = CString::new("keys").expect("keys contains no nulls");

    let object_ctor = unsafe { JS_GetPropertyStr(raw_ctx.as_ptr(), global, object_name.as_ptr()) };
    if is_exception(object_ctor) {
        return Err(JsError::Conversion {
            message: "failed to read Object constructor".to_string(),
        });
    }

    let keys_fn = unsafe { JS_GetPropertyStr(raw_ctx.as_ptr(), object_ctor, keys_name.as_ptr()) };
    if is_exception(keys_fn) {
        return Err(JsError::Conversion {
            message: "failed to read Object.keys".to_string(),
        });
    }

    unsafe {
        JS_PushArg(raw_ctx.as_ptr(), value.raw());
        JS_PushArg(raw_ctx.as_ptr(), keys_fn);
        JS_PushArg(raw_ctx.as_ptr(), js_null_value());
    }

    let keys_raw = unsafe { JS_Call(raw_ctx.as_ptr(), 1) };
    if is_exception(keys_raw) {
        return Err(JsError::Conversion {
            message: "failed to call Object.keys".to_string(),
        });
    }

    let keys_value = Value::new(raw_ctx, keys_raw);
    Vec::<String>::from_value(keys_value)
}
