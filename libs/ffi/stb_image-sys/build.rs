use std::{path::Path, process::Command};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let opt_level = std::env::var("OPT_LEVEL").unwrap();

    Command::new("clang")
        .args([
            "src/stb_image.c",
            "-c",
            &format!("-O{opt_level}"),
            "-fPIC",
            "-o",
        ])
        .arg(&format!("{out_dir}/stb_image.o"))
        .status()
        .unwrap();

    Command::new("llvm-ar")
        .args(["crus", "libstb_image.a", "stb_image.o"])
        .current_dir(Path::new(&out_dir))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static=stb_image");
    println!("cargo:rerun-if-changed=src/stb_image.c");
    println!("cargo:rerun-if-changed=src/stb_image.h");
    println!("cargo:rerun-if-changed=build.rs");
}
