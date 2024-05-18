use std::io::Write;
use std::process::Command;

const SHADER_ROOT: &str = "shaders";

#[derive(Clone, Copy)]
struct Shader {
    stage: &'static str,
    name: &'static str,
}

const SHADERS: [Shader; 3] = [
    Shader {
        stage: "vert",
        name: "basic",
    },
    Shader {
        stage: "frag",
        name: "basic",
    },
    Shader {
        stage: "comp",
        name: "display_transform",
    },
];

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let debug = match std::env::var("DEBUG").unwrap().as_str() {
        "true" | "2" | "1" => "",
        _ => "0",
    };

    let commands = SHADERS.map(|Shader { stage, name }| {
        Command::new("glslangValidator")
            .args(["--target-env", "vulkan1.3"])
            .arg("--quiet")
            .arg(&format!("-g{debug}"))
            .args(["--depfile", &format!("{out_dir}/{name}.{stage}.d")])
            .args(["-o", &format!("{out_dir}/{name}.{stage}.spv")])
            .arg(&format!("{SHADER_ROOT}/{name}.{stage}.glsl"))
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

    for Shader { stage, name } in SHADERS {
        writeln!(
            file,
            "pub const {}_{}_SPV: &[u8] = &SpirvBytes(*include_bytes!(\"{out_dir}/{name}.{stage}.spv\")).0;",
            name.to_ascii_uppercase(),
            stage.to_ascii_uppercase(),
        )
        .unwrap();
    }

    for mut command in commands {
        let status = command.wait().unwrap();
        assert!(status.success());
    }

    for Shader { stage, name } in SHADERS {
        let depfile = std::fs::read_to_string(format!("{out_dir}/{name}.{stage}.d")).unwrap();

        struct Lexer<'a> {
            bytes: &'a [u8],
            index: usize,
        }

        impl<'a> Lexer<'a> {
            fn new(bytes: &'a [u8]) -> Self {
                Self { bytes, index: 0 }
            }

            fn is_empty(&self) -> bool {
                self.index >= self.bytes.len()
            }

            fn skip(&mut self, b: u8) -> bool {
                if self.bytes.get(self.index).is_some_and(|&x| x == b) {
                    self.index += 1;
                    true
                } else {
                    false
                }
            }

            fn skip_to(&mut self, escape: u8, needle: u8) -> bool {
                let mut escape_count = 0;
                while let Some(&b) = self.bytes.get(self.index) {
                    if b == escape {
                        escape_count += 1;
                    } else if b == needle {
                        if escape_count & 1 == 0 {
                            self.index += 1;
                            return true;
                        }
                        escape_count = 0;
                    } else {
                        escape_count = 0;
                    }
                    self.index += 1;
                }
                false
            }

            fn read_to(&mut self, escape: u8, needle: u8) -> &[u8] {
                let start = self.index;
                let mut escape_count = 0;
                while let Some(&b) = self.bytes.get(self.index) {
                    if b == escape {
                        escape_count += 1;
                    } else if b == needle {
                        if escape_count & 1 == 0 {
                            break;
                        }
                        escape_count = 0;
                    } else {
                        escape_count = 0;
                    }
                    self.index += 1;
                }
                &self.bytes[start..self.index]
            }
        }

        for line in depfile.lines() {
            let mut lexer = Lexer::new(line.as_bytes());
            if lexer.skip_to(b'\\', b':') {
                lexer.skip(b' ');
                while !lexer.is_empty() {
                    let path = lexer.read_to(b'\\', b' ');
                    if let Ok(path) = std::str::from_utf8(path) {
                        println!("cargo::rerun-if-changed={path}");
                    }
                    lexer.skip(b' ');
                }
            }
        }
    }

    println!("cargo::rerun-if-changed=build.rs");
}
