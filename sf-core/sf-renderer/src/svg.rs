//! SVG parser for costume files.
//!
//! Parses basic SVG elements (path, rect, circle, ellipse, g) into a
//! structured representation. No external SVG library is used — the parser
//! is intentionally simple and test-friendly.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SvgError {
    #[error("invalid SVG: {0}")]
    InvalidSvg(String),
    #[error("parse error: {0}")]
    ParseError(String),
}

// ── Types ────────────────────────────────────────────────────────────────────

/// RGBA colour (0–255 per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvgColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SvgColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a CSS colour string. Supports `#RGB`, `#RRGGBB`, `#RRGGBBAA`,
    /// and a few named colours.
    pub fn from_css(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.starts_with('#') {
            let hex = &s[1..];
            return match hex.len() {
                3 => {
                    let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                    let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                    let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                    Some(Self::new(r, g, b, 255))
                }
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(Self::new(r, g, b, 255))
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(Self::new(r, g, b, a))
                }
                _ => None,
            };
        }
        // Named colours.
        match s {
            "red" => Some(Self::new(255, 0, 0, 255)),
            "green" => Some(Self::new(0, 128, 0, 255)),
            "blue" => Some(Self::new(0, 0, 255, 255)),
            "white" => Some(Self::new(255, 255, 255, 255)),
            "black" => Some(Self::new(0, 0, 0, 255)),
            "none" => Some(Self::new(0, 0, 0, 0)),
            _ => None,
        }
    }
}

/// Parsed SVG element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SvgElement {
    Path {
        d: String,
        fill: Option<SvgColor>,
        stroke: Option<SvgColor>,
    },
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        fill: Option<SvgColor>,
        stroke: Option<SvgColor>,
    },
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        fill: Option<SvgColor>,
        stroke: Option<SvgColor>,
    },
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        fill: Option<SvgColor>,
        stroke: Option<SvgColor>,
    },
    Group {
        children: Vec<SvgElement>,
    },
}

// ── Parsing helpers ──────────────────────────────────────────────────────────

/// Extract the value of an attribute like `fill="red"`.
fn attr_value(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = tag.find(&pattern) {
        let val_start = start + pattern.len();
        if let Some(end) = tag[val_start..].find('"') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    // Also try single quotes.
    let pattern_sq = format!("{}='", attr_name);
    if let Some(start) = tag.find(&pattern_sq) {
        let val_start = start + pattern_sq.len();
        if let Some(end) = tag[val_start..].find('\'') {
            return Some(tag[val_start..val_start + end].to_string());
        }
    }
    None
}

/// Parse a colour attribute, returning `None` if absent or unrecognised.
fn parse_color_attr(tag: &str, attr_name: &str) -> Option<SvgColor> {
    attr_value(tag, attr_name).and_then(|v| SvgColor::from_css(&v))
}

/// Parse a numeric attribute.
fn parse_float_attr(tag: &str, attr_name: &str) -> Option<f64> {
    attr_value(tag, attr_name).and_then(|v| v.parse::<f64>().ok())
}

// ── Main parser ──────────────────────────────────────────────────────────────

/// Parse an SVG string into a list of elements.
///
/// This is a minimal parser that handles the most common elements. It does
/// not implement the full SVG spec.
pub fn parse_svg(source: &str) -> Result<Vec<SvgElement>, SvgError> {
    let mut elements = Vec::new();
    let mut i = 0;
    let bytes = source.as_bytes();

    while i < source.len() {
        // Find the next tag.
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }

        // Skip comments, declarations, closing tags.
        if i + 1 < source.len() && (bytes[i + 1] == b'!' || bytes[i + 1] == b'?') {
            if let Some(end) = source[i..].find('>') {
                i += end + 1;
                continue;
            }
        }

        // Find end of tag.
        let tag_end = match source[i..].find('>') {
            Some(pos) => pos,
            None => {
                i += 1;
                continue;
            }
        };

        let tag = &source[i..i + tag_end];
        let is_self_closing = tag.ends_with('/');

        // Skip closing tags.
        if tag.starts_with("</") {
            i += tag_end + 1;
            continue;
        }

        // Extract element name.
        let name_end = tag[1..]
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(tag.len() - 1);
        let name = &tag[1..1 + name_end];

        match name {
            "path" => {
                let d = attr_value(tag, "d").unwrap_or_default();
                let fill = parse_color_attr(tag, "fill");
                let stroke = parse_color_attr(tag, "stroke");
                elements.push(SvgElement::Path { d, fill, stroke });
            }
            "rect" => {
                let x = parse_float_attr(tag, "x").unwrap_or(0.0);
                let y = parse_float_attr(tag, "y").unwrap_or(0.0);
                let w = parse_float_attr(tag, "width").unwrap_or(0.0);
                let h = parse_float_attr(tag, "height").unwrap_or(0.0);
                let fill = parse_color_attr(tag, "fill");
                let stroke = parse_color_attr(tag, "stroke");
                elements.push(SvgElement::Rect { x, y, w, h, fill, stroke });
            }
            "circle" => {
                let cx = parse_float_attr(tag, "cx").unwrap_or(0.0);
                let cy = parse_float_attr(tag, "cy").unwrap_or(0.0);
                let r = parse_float_attr(tag, "r").unwrap_or(0.0);
                let fill = parse_color_attr(tag, "fill");
                let stroke = parse_color_attr(tag, "stroke");
                elements.push(SvgElement::Circle { cx, cy, r, fill, stroke });
            }
            "ellipse" => {
                let cx = parse_float_attr(tag, "cx").unwrap_or(0.0);
                let cy = parse_float_attr(tag, "cy").unwrap_or(0.0);
                let rx = parse_float_attr(tag, "rx").unwrap_or(0.0);
                let ry = parse_float_attr(tag, "ry").unwrap_or(0.0);
                let fill = parse_color_attr(tag, "fill");
                let stroke = parse_color_attr(tag, "stroke");
                elements.push(SvgElement::Ellipse { cx, cy, rx, ry, fill, stroke });
            }
            "g" => {
                // For groups, find matching closing tag and parse children recursively.
                if !is_self_closing {
                    let close_tag = "</g>";
                    if let Some(close_pos) = source[i + tag_end + 1..].find(close_tag) {
                        let inner = &source[i + tag_end + 1..i + tag_end + 1 + close_pos];
                        let children = parse_svg(inner)?;
                        elements.push(SvgElement::Group { children });
                        i = i + tag_end + 1 + close_pos + close_tag.len();
                        continue;
                    }
                }
                elements.push(SvgElement::Group { children: vec![] });
            }
            _ => {} // svg, defs, etc. — skip
        }

        i += tag_end + 1;
    }

    Ok(elements)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rect_element() {
        let svg = r##"<svg><rect x="10" y="20" width="100" height="50" fill="#FF0000"/></svg>"##;
        let elements = parse_svg(svg).expect("should parse");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            SvgElement::Rect { x, y, w, h, fill, stroke } => {
                assert!((x - 10.0).abs() < 0.01);
                assert!((y - 20.0).abs() < 0.01);
                assert!((w - 100.0).abs() < 0.01);
                assert!((h - 50.0).abs() < 0.01);
                assert_eq!(fill, &Some(SvgColor::new(255, 0, 0, 255)));
                assert_eq!(stroke, &None);
            }
            _ => panic!("expected Rect element"),
        }
    }

    #[test]
    fn parse_circle_and_path() {
        let svg = r#"<svg>
            <circle cx="50" cy="50" r="25" fill="blue"/>
            <path d="M10 10 L20 20" stroke="black"/>
        </svg>"#;
        let elements = parse_svg(svg).expect("should parse");
        assert_eq!(elements.len(), 2);

        match &elements[0] {
            SvgElement::Circle { cx, cy, r, fill, .. } => {
                assert!((cx - 50.0).abs() < 0.01);
                assert!((cy - 50.0).abs() < 0.01);
                assert!((r - 25.0).abs() < 0.01);
                assert_eq!(fill, &Some(SvgColor::new(0, 0, 255, 255)));
            }
            _ => panic!("expected Circle"),
        }

        match &elements[1] {
            SvgElement::Path { d, stroke, .. } => {
                assert_eq!(d, "M10 10 L20 20");
                assert_eq!(stroke, &Some(SvgColor::new(0, 0, 0, 255)));
            }
            _ => panic!("expected Path"),
        }
    }

    #[test]
    fn parse_group_with_children() {
        let svg = r#"<svg><g><rect x="0" y="0" width="10" height="10" fill="red"/><circle cx="5" cy="5" r="3"/></g></svg>"#;
        let elements = parse_svg(svg).expect("should parse");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            SvgElement::Group { children } => {
                assert_eq!(children.len(), 2);
                assert!(matches!(&children[0], SvgElement::Rect { .. }));
                assert!(matches!(&children[1], SvgElement::Circle { .. }));
            }
            _ => panic!("expected Group"),
        }
    }

    #[test]
    fn parse_ellipse() {
        let svg = r##"<ellipse cx="100" cy="200" rx="50" ry="30" fill="#0fBD8C"/>"##;
        let elements = parse_svg(svg).expect("should parse");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            SvgElement::Ellipse { cx, cy, rx, ry, fill, .. } => {
                assert!((cx - 100.0).abs() < 0.01);
                assert!((cy - 200.0).abs() < 0.01);
                assert!((rx - 50.0).abs() < 0.01);
                assert!((ry - 30.0).abs() < 0.01);
                assert_eq!(fill, &Some(SvgColor::new(15, 189, 140, 255)));
            }
            _ => panic!("expected Ellipse"),
        }
    }

    #[test]
    fn svg_color_from_css() {
        assert_eq!(SvgColor::from_css("#FF0000"), Some(SvgColor::new(255, 0, 0, 255)));
        assert_eq!(SvgColor::from_css("#F00"), Some(SvgColor::new(255, 0, 0, 255)));
        assert_eq!(SvgColor::from_css("#FF000080"), Some(SvgColor::new(255, 0, 0, 128)));
        assert_eq!(SvgColor::from_css("red"), Some(SvgColor::new(255, 0, 0, 255)));
        assert_eq!(SvgColor::from_css("none"), Some(SvgColor::new(0, 0, 0, 0)));
        assert_eq!(SvgColor::from_css("unknowncolor"), None);
    }

    #[test]
    fn parse_empty_svg() {
        let elements = parse_svg("<svg></svg>").expect("should parse");
        assert!(elements.is_empty());
    }
}
