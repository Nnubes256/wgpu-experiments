use anyhow::*;
use glob::glob;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

/// An individual shader
struct ShaderData {
    /// Its source code
    src: String,
    /// Path to the source code file
    src_path: PathBuf,
    /// Path to the compiled SPIR-V file
    spv_path: PathBuf,
    /// Type of shader
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> Result<Self> {
        let extension = src_path
            .extension()
            .context("File has no extension")?
            .to_str()
            .context("Extension cannot be converted to &str")?;

        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => bail!("Unsupported shader: {}", src_path.display()),
        };

        let src = read_to_string(src_path.clone())?;
        let spv_path = src_path.with_extension(format!("{}.spv", extension));

        Ok(Self {
            src,
            src_path,
            spv_path,
            kind,
        })
    }
}

fn main() -> Result<()> {
    // Collect all shaders
    let mut shader_paths = [
        glob("./src/shaders/**/*.vert")?,
        glob("./src/shaders/**/*.frag")?,
        glob("./src/shaders/**/*.comp")?,
    ];

    let shaders = shader_paths
        .iter_mut()
        .flatten()
        .map(|glob_res| ShaderData::load(glob_res?))
        .collect::<Vec<Result<_>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    let mut compiler = shaderc::Compiler::new().context("Unable to create shader compiler")?;

    for shader in shaders {
        println!(
            "cargo:rerun-if-changed={}",
            shader.src_path.as_os_str().to_str().unwrap()
        );

        let compiled = compiler.compile_into_spirv(
            &shader.src,
            shader.kind,
            shader.src_path.to_str().unwrap(),
            "main",
            None,
        )?;

        write(shader.spv_path, compiled.as_binary_u8())?;
    }

    Ok(())
}
