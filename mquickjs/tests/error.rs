use mquickjs_rs::{Context, JsError};

#[test]
fn context_init_error_formats() {
    let err = JsError::ContextInit {
        message: "missing buffer".to_string(),
    };
    assert_eq!(format!("{err}"), "context init failed: missing buffer");
}

#[test]
fn exception_error_formats() {
    let err = JsError::Exception {
        message: "boom".to_string(),
        stack: None,
    };
    assert_eq!(format!("{err}"), "runtime error: boom");
}

#[test]
fn exception_error_formats_with_stack() {
    let err = JsError::Exception {
        message: "boom".to_string(),
        stack: Some("trace".to_string()),
    };
    assert_eq!(format!("{err}"), "runtime error: boom\ntrace");
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

#[test]
fn eval_error_captures_stack_when_available() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let err = ctx
        .eval_i32("throw new Error('boom')", "test")
        .expect_err("expected runtime error");

    match err {
        JsError::Exception { message, stack } => {
            assert!(message.contains("boom"));
            let stack = stack.expect("expected stack");
            assert!(!stack.trim().is_empty());
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn eval_error_without_stack_for_non_error() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let err = ctx
        .eval_i32("throw 'boom'", "test")
        .expect_err("expected runtime error");

    match err {
        JsError::Exception { message, stack } => {
            assert!(message.contains("boom"));
            assert!(stack.is_none());
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
