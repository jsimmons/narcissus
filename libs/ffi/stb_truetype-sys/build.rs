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
            "src/stb_truetype.c",
            "-c",
            &format!("-O{opt_level}"),
            &format!("-g{debug}"),
            "-fPIC",
            "-o",
        ])
        .arg(&format!("{out_dir}/stb_truetype.o"))
        .status()
        .unwrap();

    Command::new("llvm-ar")
        .args(["crus", "libstb_truetype.a", "stb_truetype.o"])
        .current_dir(Path::new(&out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static=stb_truetype");
    println!("cargo:rerun-if-changed=src/stb_truetype.c");
    println!("cargo:rerun-if-changed=src/stb_truetype.h");
    println!("cargo:rerun-if-changed=src/stb_rect_pack.h");
    println!("cargo:rerun-if-changed=build.rs");
}
