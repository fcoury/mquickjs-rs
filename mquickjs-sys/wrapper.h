#ifndef MQUICKJS_SYS_WRAPPER_H
#define MQUICKJS_SYS_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

#include "mquickjs.h"

extern const JSSTDLibraryDef js_stdlib;

typedef JSValue (*JSHostCallback)(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
void JS_SetHostCallback(JSHostCallback callback);

#endif /* MQUICKJS_SYS_WRAPPER_H */
