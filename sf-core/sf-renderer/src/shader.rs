//! GLSL shader definitions and introspection utilities.

// ── Sprite shaders ───────────────────────────────────────────────────────────

/// Vertex shader for sprite rendering.
pub const SPRITE_VERT: &str = r#"#version 300 es
in vec2 a_position;
in vec2 a_texcoord;
uniform mat3 u_transform;
out vec2 v_texcoord;
void main() {
    vec3 pos = u_transform * vec3(a_position, 1.0);
    gl_Position = vec4(pos.xy, 0.0, 1.0);
    v_texcoord = a_texcoord;
}"#;

/// Fragment shader for sprite rendering.
pub const SPRITE_FRAG: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
uniform sampler2D u_texture;
uniform float u_alpha;
out vec4 fragColor;
void main() {
    vec4 tex = texture(u_texture, v_texcoord);
    fragColor = vec4(tex.rgb, tex.a * u_alpha);
}"#;

// ── Pen shaders ──────────────────────────────────────────────────────────────

/// Vertex shader for pen line rendering.
pub const PEN_VERT: &str = r#"#version 300 es
in vec2 a_position;
uniform mat3 u_transform;
void main() {
    vec3 pos = u_transform * vec3(a_position, 1.0);
    gl_Position = vec4(pos.xy, 0.0, 1.0);
}"#;

/// Fragment shader for pen line rendering.
pub const PEN_FRAG: &str = r#"#version 300 es
precision mediump float;
uniform vec4 u_color;
out vec4 fragColor;
void main() {
    fragColor = u_color;
}"#;

// ── Introspection ────────────────────────────────────────────────────────────

/// Extract uniform names from a GLSL source string.
///
/// Looks for lines matching `uniform <type> <name>;` or
/// `uniform <type> <name>[<size>];`.
pub fn extract_uniforms(source: &str) -> Vec<String> {
    let mut uniforms = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("uniform ") {
            continue;
        }
        // Remove "uniform " prefix.
        let rest = &trimmed["uniform ".len()..];
        // rest is like "vec4 u_color;" or "sampler2D u_texture;" or "mat3 u_transform;"
        // Split on whitespace to skip type, then take the name (strip ; and []).
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[1].trim_end_matches(';').trim_end_matches(|c: char| c == ']')
                .split('[').next().unwrap_or("").to_string();
            if !name.is_empty() {
                uniforms.push(name);
            }
        }
    }
    uniforms
}

/// Extract attribute names from a GLSL source string.
///
/// Looks for lines matching `in <type> <name>;` (GLSL 300 es) or
/// `attribute <type> <name>;` (GLSL 100).
pub fn extract_attributes(source: &str) -> Vec<String> {
    let mut attributes = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        let rest = if trimmed.starts_with("in ") {
            Some(&trimmed["in ".len()..])
        } else if trimmed.starts_with("attribute ") {
            Some(&trimmed["attribute ".len()..])
        } else {
            None
        };

        if let Some(rest) = rest {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_end_matches(';').to_string();
                if !name.is_empty() {
                    attributes.push(name);
                }
            }
        }
    }
    attributes
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_uniforms_from_sprite_shaders() {
        let vert_uniforms = extract_uniforms(SPRITE_VERT);
        assert!(
            vert_uniforms.contains(&"u_transform".to_string()),
            "sprite vert should have u_transform, got {:?}",
            vert_uniforms
        );

        let frag_uniforms = extract_uniforms(SPRITE_FRAG);
        assert!(
            frag_uniforms.contains(&"u_texture".to_string()),
            "sprite frag should have u_texture"
        );
        assert!(
            frag_uniforms.contains(&"u_alpha".to_string()),
            "sprite frag should have u_alpha"
        );
    }

    #[test]
    fn extract_uniforms_from_pen_shaders() {
        let vert_uniforms = extract_uniforms(PEN_VERT);
        assert!(
            vert_uniforms.contains(&"u_transform".to_string()),
            "pen vert should have u_transform"
        );

        let frag_uniforms = extract_uniforms(PEN_FRAG);
        assert!(
            frag_uniforms.contains(&"u_color".to_string()),
            "pen frag should have u_color"
        );
    }

    #[test]
    fn extract_attributes_from_sprite_vert() {
        let attrs = extract_attributes(SPRITE_VERT);
        assert!(
            attrs.contains(&"a_position".to_string()),
            "sprite vert should have a_position, got {:?}",
            attrs
        );
        assert!(
            attrs.contains(&"a_texcoord".to_string()),
            "sprite vert should have a_texcoord"
        );
    }

    #[test]
    fn extract_attributes_from_pen_vert() {
        let attrs = extract_attributes(PEN_VERT);
        assert!(
            attrs.contains(&"a_position".to_string()),
            "pen vert should have a_position, got {:?}",
            attrs
        );
    }

    #[test]
    fn extract_uniforms_custom_source() {
        let src = r#"
uniform float u_time;
uniform vec2 u_resolution;
void main() {}
"#;
        let uniforms = extract_uniforms(src);
        assert_eq!(uniforms.len(), 2);
        assert!(uniforms.contains(&"u_time".to_string()));
        assert!(uniforms.contains(&"u_resolution".to_string()));
    }

    #[test]
    fn extract_attributes_legacy_glsl() {
        let src = r#"
attribute vec2 a_pos;
attribute vec4 a_color;
void main() {}
"#;
        let attrs = extract_attributes(src);
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains(&"a_pos".to_string()));
        assert!(attrs.contains(&"a_color".to_string()));
    }

    #[test]
    fn shader_sources_are_not_empty() {
        assert!(!SPRITE_VERT.is_empty());
        assert!(!SPRITE_FRAG.is_empty());
        assert!(!PEN_VERT.is_empty());
        assert!(!PEN_FRAG.is_empty());
    }
}
