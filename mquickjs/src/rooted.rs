use std::marker::PhantomData;
use std::ptr::NonNull;

use mquickjs_sys::{JS_AddGCRef, JSContext, JS_DeleteGCRef, JSGCRef};

use crate::value::Value;

/// A GC-rooted JavaScript value tied to a `Context` lifetime.
#[derive(Debug)]
pub struct RootedValue<'ctx> {
    ctx: NonNull<JSContext>,
    gc_ref: Box<JSGCRef>,
    _marker: PhantomData<&'ctx JSContext>,
}

impl<'ctx> RootedValue<'ctx> {
    pub(crate) fn new(ctx: NonNull<JSContext>, value: Value<'ctx>) -> Self {
        let mut gc_ref = Box::new(JSGCRef {
            val: 0,
            prev: std::ptr::null_mut(),
        });

        unsafe {
            let slot = JS_AddGCRef(ctx.as_ptr(), &mut *gc_ref);
            *slot = value.raw();
        }

        Self {
            ctx,
            gc_ref,
            _marker: PhantomData,
        }
    }

    /// Return the rooted value as a regular `Value`.
    pub fn to_value(&self) -> Value<'ctx> {
        Value::new(self.ctx, self.gc_ref.val)
    }
}

impl Drop for RootedValue<'_> {
    fn drop(&mut self) {
        unsafe {
            JS_DeleteGCRef(self.ctx.as_ptr(), &mut *self.gc_ref);
        }
    }
}
