use std::marker::PhantomData;
use std::ptr::NonNull;

use mquickjs_sys::{
    JSCStringBuf, JSContext, JSValue, JS_IsNumber, JS_IsString, JS_TAG_BOOL,
    JS_TAG_SPECIAL_BITS, JS_ToCStringLen, JS_ToInt32, JS_ToNumber,
};

use crate::error::JsError;

/// Opaque handle to a JavaScript value tied to a `Context`.
#[derive(Debug, Clone, Copy)]
pub struct Value<'ctx> {
    ctx: NonNull<JSContext>,
    raw: JSValue,
    _marker: PhantomData<&'ctx JSContext>,
}

impl<'ctx> Value<'ctx> {
    pub(crate) fn new(ctx: NonNull<JSContext>, raw: JSValue) -> Self {
        Self {
            ctx,
            raw,
            _marker: PhantomData,
        }
    }

    pub(crate) fn raw(&self) -> JSValue {
        self.raw
    }

    /// Convert the value to i32.
    pub fn to_i32(&self) -> Result<i32, JsError> {
        let ctx = self.ctx.as_ptr();
        let is_number = unsafe { JS_IsNumber(ctx, self.raw) };
        if is_number == 0 {
            return Err(JsError::Conversion {
                message: "expected number".to_string(),
            });
        }

        let mut out = 0i32;
        let status = unsafe { JS_ToInt32(ctx, &mut out, self.raw) };
        if status != 0 {
            return Err(JsError::Conversion {
                message: "failed to convert to i32".to_string(),
            });
        }
        Ok(out)
    }

    /// Convert the value to bool.
    pub fn to_bool(&self) -> Result<bool, JsError> {
        let tag_mask = (1u64 << JS_TAG_SPECIAL_BITS) - 1;
        let tag = (self.raw & tag_mask) as u32;
        if tag != JS_TAG_BOOL as u32 {
            return Err(JsError::Conversion {
                message: "expected bool".to_string(),
            });
        }

        Ok((self.raw >> JS_TAG_SPECIAL_BITS) != 0)
    }

    /// Convert the value to f64.
    pub fn to_f64(&self) -> Result<f64, JsError> {
        let ctx = self.ctx.as_ptr();
        let is_number = unsafe { JS_IsNumber(ctx, self.raw) };
        if is_number == 0 {
            return Err(JsError::Conversion {
                message: "expected number".to_string(),
            });
        }

        let mut out = 0f64;
        let status = unsafe { JS_ToNumber(ctx, &mut out, self.raw) };
        if status != 0 {
            return Err(JsError::Conversion {
                message: "failed to convert to f64".to_string(),
            });
        }
        Ok(out)
    }

    /// Convert the value to String.
    pub fn to_string(&self) -> Result<String, JsError> {
        let ctx = self.ctx.as_ptr();
        let is_string = unsafe { JS_IsString(ctx, self.raw) };
        if is_string == 0 {
            return Err(JsError::Conversion {
                message: "expected string".to_string(),
            });
        }

        let mut buf = JSCStringBuf { buf: [0u8; 5] };
        let mut len = 0usize;
        let ptr = unsafe { JS_ToCStringLen(ctx, &mut len, self.raw, &mut buf) };
        if ptr.is_null() {
            return Err(JsError::Conversion {
                message: "failed to convert to string".to_string(),
            });
        }

        let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}
