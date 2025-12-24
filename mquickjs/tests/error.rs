use mquickjs::JsError;

#[test]
fn context_init_error_formats() {
    let err = JsError::ContextInit {
        message: "missing buffer".to_string(),
    };
    assert_eq!(format!("{err}"), "context init failed: missing buffer");
}

#[test]
fn runtime_error_formats() {
    let err = JsError::Runtime {
        message: "boom".to_string(),
    };
    assert_eq!(format!("{err}"), "runtime error: boom");
}

#[test]
fn conversion_error_formats() {
    let err = JsError::Conversion {
        message: "bad type".to_string(),
    };
    assert_eq!(format!("{err}"), "conversion error: bad type");
}

#[test]
fn callback_error_formats() {
    let err = JsError::Callback {
        message: "oops".to_string(),
    };
    assert_eq!(format!("{err}"), "callback error: oops");
}
