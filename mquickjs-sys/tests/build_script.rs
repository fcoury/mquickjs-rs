use std::collections::HashSet;
use std::path::Path;

fn expected_lib_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "mquickjs.lib"
    } else {
        "libmquickjs.a"
    }
}

#[test]
fn build_outputs_static_library() {
    let out_dir = option_env!("MQUICKJS_SYS_OUT_DIR")
        .expect("build script must export MQUICKJS_SYS_OUT_DIR");
    let lib_path = Path::new(&out_dir).join(expected_lib_name());
    assert!(
        lib_path.exists(),
        "expected static library at {}",
        lib_path.display()
    );
}

#[test]
fn build_lists_compiled_c_sources() {
    let sources = option_env!("MQUICKJS_SYS_C_SOURCES")
        .expect("build script must export MQUICKJS_SYS_C_SOURCES");
    let basenames: HashSet<String> = sources
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| {
            Path::new(entry)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(entry)
                .to_string()
        })
        .collect();

    for expected in ["mquickjs.c", "cutils.c", "dtoa.c", "libm.c"] {
        assert!(
            basenames.contains(expected),
            "expected {} to be compiled; got {:?}",
            expected,
            basenames
        );
    }
}
