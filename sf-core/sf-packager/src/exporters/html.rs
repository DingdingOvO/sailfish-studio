//! HTML exporter.
//!
//! Generates a self-contained HTML file with:
//! - Embedded WASM runtime (base64-encoded or referenced)
//! - Inlined assets as data URIs
//! - Canvas element for rendering
//! - VM initialization script

use std::fs;
use std::path::Path;

use base64::Engine;

use crate::{PackagerConfig, PackResult, ProgressCallback, Result};
use crate::exporters::Exporter;

/// HTML exporter: creates self-contained HTML files with embedded runtime.
pub struct HtmlExporter {
    embed_runtime: bool,
    max_inline_asset_size: u64,
    minify: bool,
    title: Option<String>,
    icon: Option<String>,
}

impl HtmlExporter {
    /// Create a new HTML exporter with the given configuration.
    pub fn new(config: &PackagerConfig) -> Self {
        Self {
            embed_runtime: config.embed_runtime,
            max_inline_asset_size: config.max_inline_asset_size,
            minify: config.minify_html,
            title: config.html_title.clone(),
            icon: config.html_icon.clone(),
        }
    }
}

impl Exporter for HtmlExporter {
    fn export(
        &self,
        bundle: &crate::ProjectBundle,
        output_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<PackResult> {
        if let Some(ref cb) = progress {
            cb("preparing_html", 0, 3);
        }

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Some(ref cb) = progress {
            cb("generating_html", 1, 3);
        }

        let html = self.generate_html(bundle)?;

        if let Some(ref cb) = progress {
            cb("writing_html", 2, 3);
        }

        let content = if self.minify {
            minify_html(&html)
        } else {
            html
        };

        fs::write(output_path, &content)?;

        let size_bytes = content.len() as u64;

        Ok(PackResult {
            output_path: output_path.to_path_buf(),
            size_bytes,
            duration_ms: 0,
            checksum: None,
            format: crate::ExportFormat::Html,
            asset_count: bundle.asset_count(),
        })
    }

    fn name(&self) -> &str {
        "html"
    }

    fn extension(&self) -> &str {
        "html"
    }
}

impl HtmlExporter {
    /// Generate the HTML content for a project bundle.
    fn generate_html(&self, bundle: &crate::ProjectBundle) -> Result<String> {
        let title = self.title.as_deref().unwrap_or(&bundle.manifest.project_name);
        let icon_tag = self.icon.as_ref().map(|url| {
            format!(r#"<link rel="icon" href="{}">"#, url)
        }).unwrap_or_default();

        // Generate asset data URIs
        let asset_scripts = self.generate_asset_scripts(bundle)?;

        // Generate WASM runtime script
        let runtime_script = if self.embed_runtime {
            r#"<script>
// Sailfish WASM Runtime (embedded)
// This would contain the actual WASM runtime in production.
window.SailfishVM = {
    init: function(canvas) {
        console.log('Sailfish VM initialized on canvas:', canvas.id);
    },
    loadProject: function(source) {
        console.log('Loading project source...');
    },
    start: function() {
        console.log('Starting project...');
    },
    stop: function() {
        console.log('Stopping project...');
    }
};
</script>"#
        } else {
            r#"<script src="https://cdn.sailfish.studio/runtime/latest/sailfish.js"></script>"#
        };

        // Source code as embedded data
        let source_b64 = base64::engine::general_purpose::STANDARD.encode(bundle.source_code.as_bytes());

        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {icon_tag}
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            background: #000;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            overflow: hidden;
        }}
        #sailfish-canvas {{
            width: 480px;
            height: 360px;
            border: 1px solid #333;
            image-rendering: pixelated;
        }}
        #sailfish-loading {{
            position: absolute;
            color: #fff;
            font-family: sans-serif;
            font-size: 18px;
        }}
    </style>
</head>
<body>
    <canvas id="sailfish-canvas" width="480" height="360"></canvas>
    <div id="sailfish-loading">Loading...</div>
    {runtime_script}
    <script>
    // Project assets
    {asset_scripts}
    // Project source (base64 encoded)
    const PROJECT_SOURCE = "{source_b64}";
    // Initialize
    document.addEventListener('DOMContentLoaded', function() {{
        const canvas = document.getElementById('sailfish-canvas');
        const loading = document.getElementById('sailfish-loading');
        if (window.SailfishVM) {{
            window.SailfishVM.init(canvas);
            const source = atob(PROJECT_SOURCE);
            window.SailfishVM.loadProject(source);
            window.SailfishVM.start();
            loading.style.display = 'none';
        }} else {{
            loading.textContent = 'Error: Sailfish VM not found';
        }}
    }});
    </script>
</body>
</html>"#,
            title = html_escape(title),
            icon_tag = icon_tag,
            runtime_script = runtime_script,
            asset_scripts = asset_scripts,
            source_b64 = source_b64,
        );

        Ok(html)
    }

    /// Generate JavaScript that loads assets as data URIs.
    fn generate_asset_scripts(&self, bundle: &crate::ProjectBundle) -> Result<String> {
        let mut scripts = Vec::new();
        scripts.push("const PROJECT_ASSETS = {};".to_string());

        for (key, data) in &bundle.asset_data {
            if (data.len() as u64) <= self.max_inline_asset_size {
                let b64 = base64::engine::general_purpose::STANDARD.encode(data);
                // Find the asset info to get mime type
                let mime_type = bundle.manifest.assets.iter()
                    .find(|a| key.starts_with(&a.asset_id))
                    .map(|a| a.mime_type.as_str())
                    .unwrap_or("application/octet-stream");

                scripts.push(format!(
                    r#"PROJECT_ASSETS["{}"] = "data:{};base64,{}";"#,
                    key, mime_type, b64
                ));
            } else {
                // Large assets would be referenced externally
                scripts.push(format!(
                    "// Asset '{}' is too large to inline ({} bytes), would be referenced externally",
                    key, data.len()
                ));
            }
        }

        Ok(scripts.join("\n    "))
    }
}

/// Escape special HTML characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Simple HTML minification: remove comments, extra whitespace.
fn minify_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_comment = false;
    let mut prev_was_space = false;

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for comment start
        if i + 3 < chars.len() && chars[i] == '<' && chars[i+1] == '!' && chars[i+2] == '-' && chars[i+3] == '-' {
            in_comment = true;
            i += 4;
            continue;
        }

        // Check for comment end
        if in_comment && i + 2 < chars.len() && chars[i] == '-' && chars[i+1] == '-' && chars[i+2] == '>' {
            in_comment = false;
            i += 3;
            continue;
        }

        if in_comment {
            i += 1;
            continue;
        }

        // Collapse whitespace
        let c = chars[i];
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }

        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_bundle::ProjectBundle;
    use tempfile::tempdir;

    fn test_bundle() -> ProjectBundle {
        ProjectBundle::create_test_bundle("HTML Test")
    }

    #[test]
    fn test_html_export_creates_file() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        let result = exporter.export(&bundle, &output, None).unwrap();
        assert!(output.exists());
        assert!(result.size_bytes > 0);
        assert_eq!(result.format, crate::ExportFormat::Html);
    }

    #[test]
    fn test_html_export_contains_doctype() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_html_export_contains_title() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("<title>HTML Test</title>"));
    }

    #[test]
    fn test_html_export_custom_title() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let mut config = PackagerConfig::default();
        config.html_title = Some("Custom Title".to_string());
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("<title>Custom Title</title>"));
    }

    #[test]
    fn test_html_export_contains_canvas() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("sailfish-canvas"));
    }

    #[test]
    fn test_html_export_embeds_runtime() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let mut config = PackagerConfig::default();
        config.embed_runtime = true;
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("SailfishVM"));
    }

    #[test]
    fn test_html_export_external_runtime() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let mut config = PackagerConfig::default();
        config.embed_runtime = false;
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("cdn.sailfish.studio"));
    }

    #[test]
    fn test_html_export_contains_base64_source() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("PROJECT_SOURCE"));
        assert!(content.contains("atob("));
    }

    #[test]
    fn test_html_export_contains_assets_as_data_uris() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("PROJECT_ASSETS"));
        assert!(content.contains("data:image/svg+xml;base64,"));
    }

    #[test]
    fn test_html_export_with_progress() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        let cb: ProgressCallback = Box::new(|_stage, _current, _total| {});
        let result = exporter.export(&bundle, &output, Some(cb)).unwrap();
        assert!(result.size_bytes > 0);
    }

    #[test]
    fn test_html_export_minified() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let mut config = PackagerConfig::default();
        config.minify_html = true;
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        // Minified should be smaller but still valid
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("sailfish-canvas"));
    }

    #[test]
    fn test_html_export_large_asset_not_inlined() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let mut config = PackagerConfig::default();
        config.max_inline_asset_size = 10; // Very small limit
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        // Large asset should be commented out
        assert!(content.contains("too large to inline") || content.contains("referenced externally"));
    }

    #[test]
    fn test_html_export_custom_icon() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let mut config = PackagerConfig::default();
        config.html_icon = Some("https://example.com/icon.png".to_string());
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains(r#"href="https://example.com/icon.png""#));
    }

    #[test]
    fn test_html_exporter_name_and_extension() {
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        assert_eq!(exporter.name(), "html");
        assert_eq!(exporter.extension(), "html");
    }

    #[test]
    fn test_html_export_creates_parent_dirs() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("nested/dir/test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = test_bundle();

        exporter.export(&bundle, &output, None).unwrap();
        assert!(output.exists());
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("hello"), "hello");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_minify_html_removes_comments() {
        let html = "<!-- comment -->hello<!-- another -->";
        let minified = minify_html(html);
        assert!(!minified.contains("comment"));
        assert!(minified.contains("hello"));
    }

    #[test]
    fn test_minify_html_collapses_whitespace() {
        let html = "<div>   hello   world   </div>";
        let minified = minify_html(html);
        // Should have collapsed spaces
        assert!(minified.contains("hello"));
        assert!(minified.contains("world"));
        assert!(minified.len() < html.len());
    }

    #[test]
    fn test_minify_html_preserves_structure() {
        let html = "<div><p>hello</p></div>";
        let minified = minify_html(html);
        assert!(minified.contains("<div>"));
        assert!(minified.contains("</div>"));
        assert!(minified.contains("<p>"));
        assert!(minified.contains("hello"));
    }

    #[test]
    fn test_html_export_empty_bundle() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = ProjectBundle::new("Empty Project");

        let result = exporter.export(&bundle, &output, None).unwrap();
        assert!(output.exists());
        assert!(result.size_bytes > 0);

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("Empty Project"));
    }

    #[test]
    fn test_html_export_with_special_chars_in_name() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("test.html");
        let config = PackagerConfig::default();
        let exporter = HtmlExporter::new(&config);
        let bundle = ProjectBundle::new("Test <Project> & \"Stuff\"");

        exporter.export(&bundle, &output, None).unwrap();
        let content = fs::read_to_string(&output).unwrap();
        // Should be escaped
        assert!(content.contains("&lt;Project&gt;"));
        assert!(content.contains("&amp;"));
    }
}
