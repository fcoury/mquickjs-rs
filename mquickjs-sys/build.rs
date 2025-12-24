use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn mquickjs_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..").join("..").join("mquickjs")
}

fn host_exe_suffix(host: &str) -> &'static str {
    if host.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

fn build_mqjs_stdlib(mquickjs_dir: &Path, out_dir: &Path, host: &str) -> PathBuf {
    let exe_path = out_dir.join(format!("mqjs_stdlib{}", host_exe_suffix(host)));

    let mut build = cc::Build::new();
    build.host(host).target(host);
    build.include(mquickjs_dir);
    build.warnings(false);
    build.flag_if_supported("-std=c99");
    build.flag_if_supported("-O2");

    let compiler = build.get_compiler();
    let mut cmd = compiler.to_command();
    if compiler.is_like_msvc() {
        cmd.arg(format!("/Fe:{}", exe_path.display()));
    } else {
        cmd.arg("-o").arg(&exe_path);
    }

    cmd.arg(mquickjs_dir.join("mqjs_stdlib.c"));
    cmd.arg(mquickjs_dir.join("mquickjs_build.c"));

    let status = cmd.status().expect("failed to build mqjs_stdlib");
    if !status.success() {
        panic!("mqjs_stdlib build failed with status {status}");
    }

    exe_path
}

fn generate_atom_header(
    mqjs_stdlib: &Path,
    out_dir: &Path,
    target_pointer_width: &str,
) -> PathBuf {
    let mut cmd = Command::new(mqjs_stdlib);
    cmd.arg("-a");
    if target_pointer_width == "32" {
        cmd.arg("-m32");
    }

    let output = cmd.output().expect("failed to run mqjs_stdlib");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("mqjs_stdlib failed: {stderr}");
    }

    let header_path = out_dir.join("mquickjs_atom.h");
    std::fs::write(&header_path, output.stdout).expect("failed to write mquickjs_atom.h");
    header_path
}

fn generate_stdlib_source(
    mqjs_stdlib: &Path,
    out_dir: &Path,
    target_pointer_width: &str,
) -> PathBuf {
    let mut cmd = Command::new(mqjs_stdlib);
    if target_pointer_width == "32" {
        cmd.arg("-m32");
    }

    let output = cmd.output().expect("failed to run mqjs_stdlib");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("mqjs_stdlib failed: {stderr}");
    }

    let source_path = out_dir.join("mquickjs_stdlib.c");
    let mut contents = String::from("#include <stddef.h>\n#include <stdint.h>\n#include \"mquickjs_priv.h\"\n\n");
    contents.push_str(
        "JSValue js_print(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\
JSValue js_gc(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\
JSValue js_date_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\
JSValue js_performance_now(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\
JSValue js_load(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\
JSValue js_setTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\
JSValue js_clearTimeout(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);\n\n",
    );
    contents.push_str(&String::from_utf8_lossy(&output.stdout));
    std::fs::write(&source_path, contents).expect("failed to write mquickjs_stdlib.c");
    source_path
}

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let mquickjs_dir = mquickjs_dir(&manifest_dir);

    let sources = ["mquickjs.c", "cutils.c", "dtoa.c", "libm.c"];
    for source in &sources {
        println!(
            "cargo:rerun-if-changed={}",
            mquickjs_dir.join(source).display()
        );
    }
    for header in ["mquickjs.h", "mquickjs_priv.h", "mquickjs_build.h", "cutils.h", "libm.h"] {
        println!(
            "cargo:rerun-if-changed={}",
            mquickjs_dir.join(header).display()
        );
    }
    for generator in ["mqjs_stdlib.c", "mquickjs_build.c"] {
        println!(
            "cargo:rerun-if-changed={}",
            mquickjs_dir.join(generator).display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("src").join("stdlib_stubs.c").display()
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let wrapper = manifest_dir.join("wrapper.h");
    println!("cargo:rerun-if-changed={}", wrapper.display());
    println!("cargo:rustc-env=MQUICKJS_SYS_OUT_DIR={}", out_dir.display());
    let joined_sources = sources
        .iter()
        .map(|source| mquickjs_dir.join(source).display().to_string())
        .collect::<Vec<_>>()
        .join(";");
    println!("cargo:rustc-env=MQUICKJS_SYS_C_SOURCES={}", joined_sources);

    let host = env::var("HOST").expect("HOST not set");
    let target_pointer_width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
        .unwrap_or_else(|_| "64".to_string());

    let mqjs_stdlib = build_mqjs_stdlib(&mquickjs_dir, &out_dir, &host);
    let atom_header = generate_atom_header(&mqjs_stdlib, &out_dir, &target_pointer_width);
    let stdlib_source = generate_stdlib_source(&mqjs_stdlib, &out_dir, &target_pointer_width);
    println!("cargo:rerun-if-changed={}", atom_header.display());
    println!("cargo:rerun-if-changed={}", stdlib_source.display());

    let bindings = bindgen::Builder::default()
        .header(wrapper.to_string_lossy())
        .clang_arg(format!("-I{}", mquickjs_dir.display()))
        .clang_arg(format!("-I{}", out_dir.display()))
        .allowlist_type("JS.*")
        .allowlist_function("JS_.*")
        .allowlist_var("JS_.*")
        .allowlist_var("js_stdlib")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("could not write bindings");

    let mut build = cc::Build::new();
    build.include(&mquickjs_dir);
    build.include(&out_dir);
    build.warnings(false);

    for source in &sources {
        build.file(mquickjs_dir.join(source));
    }
    build.file(&stdlib_source);
    build.file(manifest_dir.join("src").join("stdlib_stubs.c"));

    let compiler = build.get_compiler();
    if compiler.is_like_msvc() {
        build.define("_CRT_SECURE_NO_WARNINGS", None);
        build.flag_if_supported("/std:c11");
    } else {
        build.flag_if_supported("-std=c99");
        build.flag_if_supported("-O2");
    }

    build.compile("mquickjs");
}
