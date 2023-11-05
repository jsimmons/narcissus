use std::{path::Path, process::Command};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let opt_level = std::env::var("OPT_LEVEL").unwrap();
    let debug = match std::env::var("DEBUG").unwrap().as_str() {
        "true" | "2" => "",
        "1" => "line-tables-only",
        _ => "0",
    };

    Command::new("clang")
        .args([
            "src/sqlite3.c",
            "-c",
            &format!("-O{opt_level}"),
            &format!("-g{debug}"),
            "-DSQLITE_DQS=0",
            "-DSQLITE_DEFAULT_WAL_SYNCHRONOUS=1",
            "-DSQLITE_LIKE_DOESNT_MATCH_BLOBS",
            "-DSQLITE_MAX_EXPR_DEPTH=0",
            "-DSQLITE_OMIT_DECLTYPE",
            "-DSQLITE_OMIT_DEPRECATED",
            "-DSQLITE_OMIT_PROGRESS_CALLBACK",
            "-DSQLITE_OMIT_SHARED_CACHE",
            "-DSQLITE_USE_ALLOCA",
            "-DSQLITE_OMIT_AUTOINIT",
            "-DSQLITE_THREADSAFE=2",
            "-fPIC",
            "-o",
        ])
        .arg(&format!("{out_dir}/sqlite3.o"))
        .status()
        .unwrap();

    Command::new("llvm-ar")
        .args(["crus", "libsqlite3.a", "sqlite3.o"])
        .current_dir(Path::new(&out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static=sqlite3");
    println!("cargo:rerun-if-changed=src/sqlite3.c");
    println!("cargo:rerun-if-changed=src/sqlite3.h");
    println!("cargo:rerun-if-changed=build.rs");
}
