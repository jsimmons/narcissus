use std::io::Write;
use std::process::Command;

const SHADER_ROOT: &str = "shaders";

#[derive(Clone, Copy)]
enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    fn name(self) -> &'static str {
        match self {
            ShaderStage::Vertex => "vert",
            ShaderStage::Fragment => "frag",
        }
    }
}

#[derive(Clone, Copy)]
struct Shader {
    stage: ShaderStage,
    name: &'static str,
}

const SHADERS: [Shader; 4] = [
    Shader {
        stage: ShaderStage::Vertex,
        name: "basic",
    },
    Shader {
        stage: ShaderStage::Fragment,
        name: "basic",
    },
    Shader {
        stage: ShaderStage::Vertex,
        name: "text",
    },
    Shader {
        stage: ShaderStage::Fragment,
        name: "text",
    },
];

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let debug = match std::env::var("DEBUG").unwrap().as_str() {
        "true" | "2" | "1" => "",
        _ => "0",
    };

    let commands = SHADERS.map(|shader| {
        Command::new("glslang")
            .args(["--target-env", "vulkan1.3"])
            .arg(&format!("-g{debug}"))
            .args([
                "-o",
                &format!("{out_dir}/{}.{}.spv", shader.name, shader.stage.name()),
            ])
            .arg(&format!(
                "{SHADER_ROOT}/{}.{}.glsl",
                shader.name,
                shader.stage.name()
            ))
            .spawn()
            .unwrap()
    });

    let dst_path = std::path::Path::new(&out_dir).join("shaders.rs");
    let mut file = std::fs::File::create(dst_path).unwrap();

    writeln!(
        file,
        "#[repr(align(4))]\nstruct SpirvBytes<const LEN: usize>([u8; LEN]);"
    )
    .unwrap();

    for shader in SHADERS {
        writeln!(
            file,
            "pub const {}_{}_SPV: &'static [u8] = &SpirvBytes(*include_bytes!(\"{out_dir}/{}.{}.spv\")).0;",
            shader.name.to_ascii_uppercase(),
            shader.stage.name().to_ascii_uppercase(),
            shader.name,
            shader.stage.name()
        )
        .unwrap();
    }

    for mut command in commands {
        let status = command.wait().unwrap();
        assert!(status.success());
    }

    for shader in SHADERS {
        println!(
            "cargo::rerun-if-changed={SHADER_ROOT}/{}.{}.glsl",
            shader.name,
            shader.stage.name()
        )
    }

    println!("cargo::rerun-if-changed=build.rs");
}
