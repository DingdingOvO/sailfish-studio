//! WebGL2 context initialization and shader compilation.
//!
//! This module works purely with data structs — no actual GL context is
//! required, making it fully testable without a GPU.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum GlError {
    #[error("empty vertex shader source")]
    EmptyVertexSource,
    #[error("empty fragment shader source")]
    EmptyFragmentSource,
    #[error("GLSL validation error in {stage}: {message}")]
    GlslValidation { stage: String, message: String },
}

// ── Types ────────────────────────────────────────────────────────────────────

/// Configuration for creating a WebGL2 context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGLConfig {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub antialias: bool,
    pub alpha: bool,
    pub premultiplied_alpha: bool,
}

impl Default for WebGLConfig {
    fn default() -> Self {
        Self {
            canvas_width: 480,
            canvas_height: 360,
            antialias: true,
            alpha: true,
            premultiplied_alpha: false,
        }
    }
}

/// Shader compilation stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

/// Metadata about a compiled shader program (pure data — no GL objects).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderInfo {
    pub vertex_source: String,
    pub fragment_source: String,
    pub uniforms: Vec<String>,
    pub attributes: Vec<String>,
}

// ── Shader compilation (data-level) ──────────────────────────────────────────

/// "Compile" a shader pair by validating sources and extracting metadata.
pub fn compile_shader_config(vertex: &str, fragment: &str) -> Result<ShaderInfo, GlError> {
    if vertex.trim().is_empty() {
        return Err(GlError::EmptyVertexSource);
    }
    if fragment.trim().is_empty() {
        return Err(GlError::EmptyFragmentSource);
    }

    validate_glsl(vertex, ShaderStage::Vertex)?;
    validate_glsl(fragment, ShaderStage::Fragment)?;

    let uniforms = crate::shader::extract_uniforms(vertex)
        .into_iter()
        .chain(crate::shader::extract_uniforms(fragment))
        .collect();
    let attributes = crate::shader::extract_attributes(vertex);

    Ok(ShaderInfo {
        vertex_source: vertex.to_string(),
        fragment_source: fragment.to_string(),
        uniforms,
        attributes,
    })
}

/// Basic GLSL syntax validation.
///
/// Checks that:
/// - The source contains a `void main()` function definition.
/// - The source is not just whitespace.
pub fn validate_glsl(source: &str, stage: ShaderStage) -> Result<(), GlError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(GlError::GlslValidation {
            stage: format!("{:?}", stage),
            message: "source is empty".to_string(),
        });
    }

    // Must contain a main function.
    if !trimmed.contains("void main(") && !trimmed.contains("void main (") {
        return Err(GlError::GlslValidation {
            stage: format!("{:?}", stage),
            message: "missing void main() function".to_string(),
        });
    }

    Ok(())
}

/// Default vertex shader source for sprite rendering.
pub fn default_sprite_vert() -> &'static str {
    crate::shader::SPRITE_VERT
}

/// Default fragment shader source for sprite rendering.
pub fn default_sprite_frag() -> &'static str {
    crate::shader::SPRITE_FRAG
}

/// Default vertex shader source for pen line rendering.
pub fn default_pen_vert() -> &'static str {
    crate::shader::PEN_VERT
}

/// Default fragment shader source for pen line rendering.
pub fn default_pen_frag() -> &'static str {
    crate::shader::PEN_FRAG
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webgl_config_defaults() {
        let cfg = WebGLConfig::default();
        assert_eq!(cfg.canvas_width, 480);
        assert_eq!(cfg.canvas_height, 360);
        assert!(cfg.antialias);
        assert!(cfg.alpha);
        assert!(!cfg.premultiplied_alpha);
    }

    #[test]
    fn compile_shader_config_with_valid_sources() {
        let vert = r#"#version 300 es
in vec2 a_position;
void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
}"#;
        let frag = r#"#version 300 es
precision mediump float;
uniform vec4 u_color;
out vec4 fragColor;
void main() {
    fragColor = u_color;
}"#;

        let info = compile_shader_config(vert, frag).expect("should compile");
        assert_eq!(info.vertex_source, vert);
        assert_eq!(info.fragment_source, frag);
        // Should extract "u_color" from fragment.
        assert!(
            info.uniforms.contains(&"u_color".to_string()),
            "should find u_color uniform"
        );
        // Should extract "a_position" from vertex.
        assert!(
            info.attributes.contains(&"a_position".to_string()),
            "should find a_position attribute"
        );
    }

    #[test]
    fn compile_shader_config_rejects_empty_vertex() {
        let result = compile_shader_config("", "void main() {}");
        assert!(
            matches!(result, Err(GlError::EmptyVertexSource)),
            "should reject empty vertex source"
        );
    }

    #[test]
    fn compile_shader_config_rejects_empty_fragment() {
        let result = compile_shader_config("void main() {}", "");
        assert!(
            matches!(result, Err(GlError::EmptyFragmentSource)),
            "should reject empty fragment source"
        );
    }

    #[test]
    fn validate_glsl_rejects_missing_main() {
        let src = "uniform float u_time;";
        let result = validate_glsl(src, ShaderStage::Vertex);
        assert!(result.is_err(), "should reject shader without main()");
        if let Err(GlError::GlslValidation { stage, message }) = result {
            assert_eq!(stage, "Vertex");
            assert!(message.contains("main"));
        } else {
            panic!("wrong error type");
        }
    }

    #[test]
    fn validate_glsl_accepts_valid_source() {
        let src = "void main() { gl_Position = vec4(1.0); }";
        assert!(
            validate_glsl(src, ShaderStage::Vertex).is_ok(),
            "should accept valid source"
        );
    }

    #[test]
    fn default_shaders_are_valid() {
        // Verify that the default shader sources pass validation.
        assert!(
            validate_glsl(default_sprite_vert(), ShaderStage::Vertex).is_ok(),
            "sprite vertex shader should be valid"
        );
        assert!(
            validate_glsl(default_sprite_frag(), ShaderStage::Fragment).is_ok(),
            "sprite fragment shader should be valid"
        );
        assert!(
            validate_glsl(default_pen_vert(), ShaderStage::Vertex).is_ok(),
            "pen vertex shader should be valid"
        );
        assert!(
            validate_glsl(default_pen_frag(), ShaderStage::Fragment).is_ok(),
            "pen fragment shader should be valid"
        );
    }
}
