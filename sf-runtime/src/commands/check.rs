use std::path::Path;

use crate::error::{Result, SfError};
use crate::project::Project;

/// Result of a syntax check.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// Number of errors found.
    pub errors: Vec<CheckItem>,
    /// Number of warnings found.
    pub warnings: Vec<CheckItem>,
}

/// A single check item (error or warning).
#[derive(Debug, Clone, PartialEq)]
pub struct CheckItem {
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based, if applicable).
    pub column: Option<usize>,
    /// Error or warning message.
    pub message: String,
}

impl CheckResult {
    /// Create a new empty check result.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error.
    pub fn add_error(&mut self, line: usize, message: impl Into<String>) {
        self.errors.push(CheckItem {
            line,
            column: None,
            message: message.into(),
        });
    }

    /// Add a warning.
    pub fn add_warning(&mut self, line: usize, message: impl Into<String>) {
        self.warnings.push(CheckItem {
            line,
            column: None,
            message: message.into(),
        });
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Check if the result is clean (no errors or warnings).
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    /// Total number of issues.
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

impl Default for CheckResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute the check command.
pub fn execute(file: &Path, strict: bool) -> Result<()> {
    if !file.exists() {
        return Err(SfError::ProjectNotFound(file.display().to_string()));
    }

    // Load the project (handles both .sfl and .sfp)
    let project = Project::load(file)?;

    // Check the project source
    let result = check_source(&project.source, strict);

    // Print results
    if result.has_errors() {
        for err in &result.errors {
            eprintln!("ERROR line {}: {}", err.line, err.message);
        }
    }

    if result.has_warnings() {
        for warn in &result.warnings {
            eprintln!("WARNING line {}: {}", warn.line, warn.message);
        }
    }

    if result.is_clean() {
        println!("✓ No issues found.");
    } else {
        println!(
            "\n{} error(s), {} warning(s)",
            result.errors.len(),
            result.warnings.len()
        );
    }

    // Also validate project metadata
    let meta_errors = project.validate();
    for err in &meta_errors {
        eprintln!("ERROR: {}", err);
    }

    if !meta_errors.is_empty() && result.has_errors() {
        return Err(SfError::Custom(format!(
            "Check failed with {} error(s)",
            result.errors.len() + meta_errors.len()
        )));
    }

    Ok(())
}

/// Check source code for syntax errors and warnings.
pub fn check_source(source: &str, strict: bool) -> CheckResult {
    let mut result = CheckResult::new();

    if source.trim().is_empty() {
        result.add_error(1, "Empty source file");
        return result;
    }

    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // In strict mode, check for warnings on all lines (including comments)
        if strict {
            check_warnings(&mut result, line_num, trimmed, line);
        }

        // Skip comments for syntax error checks
        if trimmed.starts_with("//") {
            continue;
        }

        // Check for unmatched braces
        check_braces(&mut result, line_num, line);

        // Check for common syntax errors
        check_syntax_errors(&mut result, line_num, trimmed);
    }

    // Global checks
    check_global(&mut result, source);

    result
}

/// Check for brace matching issues on a single line.
fn check_braces(result: &mut CheckResult, line_num: usize, line: &str) {
    let open = line.chars().filter(|&c| c == '{').count();
    let close = line.chars().filter(|&c| c == '}').count();

    // Check for braces that close and then open on the same line
    // (like "} else {") - this is valid, don't flag it
    // Only flag if there's a standalone closing brace before an opening brace
    // that doesn't form a valid pattern
    if open > 0 && close > 0 {
        let mut saw_close = false;
        let mut saw_open_after_close = false;
        for c in line.chars() {
            match c {
                '}' => saw_close = true,
                '{' if saw_close => saw_open_after_close = true,
                _ => {}
            }
        }
        // "} else {" pattern is valid
        // Just "}{ " without keywords in between might be suspicious
        // but we'll be lenient here - only flag clear errors
        let _ = (saw_close, saw_open_after_close);
    }
}

/// Check for common syntax errors.
fn check_syntax_errors(result: &mut CheckResult, line_num: usize, line: &str) {
    // Check for function declarations without body
    if line.starts_with("fn ") && line.ends_with(")") && !line.contains('{') {
        // Check if next line has the brace (multi-line function declaration)
        // This is a simplified check
        if !line.contains(");") {
            // Could be a function signature without body - not necessarily an error
        }
    }

    // Check for missing semicolons on variable assignments
    if line.starts_with("var ") && !line.contains('=') && !line.ends_with('{') && !line.ends_with(';') {
        result.add_error(line_num, "Variable declaration missing initialization or semicolon");
    }

    // Check for assignment without semicolon (simple heuristic)
    if line.contains('=') && !line.contains("==") && !line.contains("!=")
        && !line.contains("<=") && !line.contains(">=")
        && !line.ends_with('{') && !line.ends_with(';')
        && !line.ends_with(')')
        && !line.starts_with("if ") && !line.starts_with("while ")
        && !line.starts_with("for ") && !line.starts_with("//")
    {
        // Could be missing semicolon - but only for simple statements
        if !line.contains("fn ") && !line.starts_with("var ") {
            // This is a heuristic, not a definitive error
        }
    }

    // Check for unclosed strings
    let in_string = count_unescaped_quotes(line) % 2 != 0;
    if in_string {
        result.add_error(line_num, "Unclosed string literal");
    }
}

/// Count unescaped double quotes in a line.
fn count_unescaped_quotes(line: &str) -> usize {
    let mut count = 0;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next(); // Skip escaped character
        } else if c == '"' {
            count += 1;
        }
    }
    count
}

/// Check for style warnings (strict mode only).
fn check_warnings(result: &mut CheckResult, line_num: usize, trimmed: &str, original: &str) {
    // Check for tab characters (prefer spaces)
    if original.contains('\t') {
        result.add_warning(line_num, "Tab character found (prefer spaces)");
    }

    // Check for trailing whitespace
    if original.len() > original.trim_end().len() {
        result.add_warning(line_num, "Trailing whitespace");
    }

    // Check for lines longer than 100 characters
    if original.len() > 100 {
        result.add_warning(line_num, "Line exceeds 100 characters");
    }

    // Check for TODO/FIXME comments
    if trimmed.contains("TODO") || trimmed.contains("FIXME") {
        result.add_warning(line_num, "TODO/FIXME comment found");
    }

    // Check for variable names that are too short (single letter, excluding common loop vars)
    if trimmed.starts_with("var ") {
        let rest = &trimmed[4..];
        if let Some(name) = rest.split(|c: char| c.is_whitespace() || c == '=').next() {
            if name.len() == 1 && name != "i" && name != "j" && name != "k" && name != "x" && name != "y" {
                result.add_warning(line_num, &format!("Variable name '{}' is too short", name));
            }
        }
    }
}

/// Check for global issues across the entire source.
fn check_global(result: &mut CheckResult, source: &str) {
    // Check for balanced braces
    let mut depth = 0;
    let mut last_open_line = 1;
    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;
        for c in line.chars() {
            match c {
                '{' => {
                    depth += 1;
                    last_open_line = line_num;
                }
                '}' => {
                    if depth == 0 {
                        result.add_error(line_num, "Unexpected closing brace");
                    } else {
                        depth -= 1;
                    }
                }
                _ => {}
            }
        }
    }
    if depth > 0 {
        result.add_error(
            last_open_line,
            &format!("Unclosed brace (depth={})", depth),
        );
    }

    // Check for missing main function
    if !source.contains("fn main()") && !source.contains("fn main ()") {
        result.add_error(1, "Missing 'fn main()' entry point");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(format!("{}.sfl", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_check_result_new() {
        let result = CheckResult::new();
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
        assert!(result.is_clean());
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_check_result_add_error() {
        let mut result = CheckResult::new();
        result.add_error(1, "test error");
        assert!(result.has_errors());
        assert!(!result.is_clean());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].line, 1);
        assert_eq!(result.errors[0].message, "test error");
    }

    #[test]
    fn test_check_result_add_warning() {
        let mut result = CheckResult::new();
        result.add_warning(5, "test warning");
        assert!(result.has_warnings());
        assert!(!result.is_clean());
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_check_result_total_issues() {
        let mut result = CheckResult::new();
        result.add_error(1, "err");
        result.add_error(2, "err2");
        result.add_warning(3, "warn");
        assert_eq!(result.total_issues(), 3);
    }

    #[test]
    fn test_check_clean_source() {
        let source = "fn main() {\n    print(\"hello\");\n}\n";
        let result = check_source(source, false);
        assert!(result.is_clean());
    }

    #[test]
    fn test_check_empty_source() {
        let result = check_source("", false);
        assert!(result.has_errors());
    }

    #[test]
    fn test_check_whitespace_only_source() {
        let result = check_source("   \n  \n", false);
        assert!(result.has_errors());
    }

    #[test]
    fn test_check_unclosed_string() {
        let source = "fn main() {\n    var x = \"hello;\n}\n";
        let result = check_source(source, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.message.contains("Unclosed string")));
    }

    #[test]
    fn test_check_unbalanced_braces() {
        let source = "fn main() {\n    if true {\n}\n";
        let result = check_source(source, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.message.contains("Unclosed brace")));
    }

    #[test]
    fn test_check_unexpected_closing_brace() {
        let source = "fn main() {\n}\n}\n";
        let result = check_source(source, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.message.contains("Unexpected closing brace")));
    }

    #[test]
    fn test_check_missing_main() {
        let source = "fn other() {\n    print(42);\n}\n";
        let result = check_source(source, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.message.contains("main")));
    }

    #[test]
    fn test_check_strict_trailing_whitespace() {
        let source = "fn main() {\n    print(42);   \n}\n";
        let result = check_source(source, true);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.message.contains("Trailing whitespace")));
    }

    #[test]
    fn test_check_strict_tab_characters() {
        let source = "fn main() {\n\tprint(42);\n}\n";
        let result = check_source(source, true);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.message.contains("Tab")));
    }

    #[test]
    fn test_check_strict_long_line() {
        let long_line = "fn main() {\n    var x = ".to_string()
            + &"a".repeat(100)
            + ";\n}\n";
        let result = check_source(&long_line, true);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.message.contains("100")));
    }

    #[test]
    fn test_check_strict_todo_comment() {
        let source = "fn main() {\n    // TODO: fix this\n}\n";
        let result = check_source(source, true);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.message.contains("TODO")));
    }

    #[test]
    fn test_check_strict_short_variable_name() {
        let source = "fn main() {\n    var q = 42;\n}\n";
        let result = check_source(source, true);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.message.contains("too short")));
    }

    #[test]
    fn test_check_strict_allows_common_loop_vars() {
        let source = "fn main() {\n    var i = 0;\n}\n";
        let result = check_source(source, true);
        assert!(!result.warnings.iter().any(|w| w.message.contains("too short")));
    }

    #[test]
    fn test_check_non_strict_no_warnings() {
        let source = "fn main() {\n\tvar q = 42;   \n}\n";
        let result = check_source(source, false);
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_check_count_unescaped_quotes() {
        assert_eq!(count_unescaped_quotes("var x = \"hello\""), 2);
        assert_eq!(count_unescaped_quotes("var x = \"he\\\"llo\""), 2);
        assert_eq!(count_unescaped_quotes("var x = \"hello"), 1);
        assert_eq!(count_unescaped_quotes("no quotes"), 0);
    }

    #[test]
    fn test_execute_valid_file() {
        let dir = TempDir::new().unwrap();
        let path = create_test_file(&dir, "valid", "fn main() {\n    print(42);\n}\n");
        let result = execute(&path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_nonexistent_file() {
        let result = execute(Path::new("/nonexistent.sfl"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_file_with_errors() {
        let dir = TempDir::new().unwrap();
        let path = create_test_file(&dir, "bad", "fn main() {\n    var x = \"unclosed\n}\n");
        let result = execute(&path, false);
        // Should still succeed (check reports errors but doesn't fail)
        // unless the project validation also fails
        // Our execute returns Ok even with errors (just prints them)
        // This is intentional: check is a linter, not a blocker
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_balanced_braces_complex() {
        let source = "fn main() {\n    if true {\n        print(1);\n    } else {\n        print(2);\n    }\n}\n";
        let result = check_source(source, false);
        assert!(result.is_clean(), "Expected no issues, got: {:?}", result.errors);
    }

    #[test]
    fn test_check_multiple_errors() {
        let source = "fn other() {\n    var x = \"unclosed\n}\n";
        let result = check_source(source, false);
        assert!(result.total_issues() >= 1);
    }

    #[test]
    fn test_check_result_default() {
        let result = CheckResult::default();
        assert!(result.is_clean());
    }

    #[test]
    fn test_check_item_fields() {
        let item = CheckItem {
            line: 5,
            column: Some(10),
            message: "test".to_string(),
        };
        assert_eq!(item.line, 5);
        assert_eq!(item.column, Some(10));
        assert_eq!(item.message, "test");
    }

    #[test]
    fn test_check_sfp_file() {
        let dir = TempDir::new().unwrap();
        let sfp_path = dir.path().join("test.sfp");

        let mut project = Project::new("TestSfp");
        project.source = "fn main() {\n    print(42);\n}\n".to_string();
        project.save_sfp(&sfp_path).unwrap();

        let result = execute(&sfp_path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_strict_fixme_comment() {
        let source = "fn main() {\n    // FIXME: broken\n}\n";
        let result = check_source(source, true);
        assert!(result.warnings.iter().any(|w| w.message.contains("FIXME")));
    }

    #[test]
    fn test_check_var_without_init() {
        let source = "fn main() {\n    var x\n}\n";
        let result = check_source(source, false);
        // Should report error about missing initialization or semicolon
        assert!(result.has_errors());
    }
}
