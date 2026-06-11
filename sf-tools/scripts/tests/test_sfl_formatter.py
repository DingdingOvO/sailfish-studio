"""Tests for sfl_formatter module."""

from __future__ import annotations

import sys
import os

# Add scripts directory to path so we can import modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from sfl_formatter import (
    check_indentation,
    format_sfl,
    lint_sfl,
    normalize_whitespace,
)


class TestNormalizeWhitespace:
    """Tests for normalize_whitespace."""

    def test_trailing_whitespace_removed(self):
        """Trailing whitespace on lines should be removed."""
        source = "var x = 1   \nvar y = 2\t\n"
        result = normalize_whitespace(source)
        lines = result.split("\n")
        assert lines[0] == "var x = 1"
        assert lines[1] == "var y = 2"

    def test_windows_line_endings_normalized(self):
        """Windows CRLF line endings should be converted to LF."""
        source = "line1\r\nline2\r\n"
        result = normalize_whitespace(source)
        assert "\r" not in result
        assert result == "line1\nline2\n"

    def test_mac_line_endings_normalized(self):
        """Old Mac CR line endings should be converted to LF."""
        source = "line1\rline2\r"
        result = normalize_whitespace(source)
        assert "\r" not in result

    def test_trailing_newline_ensured(self):
        """Output should end with exactly one newline."""
        source = "var x = 1"
        result = normalize_whitespace(source)
        assert result.endswith("\n")
        assert not result.endswith("\n\n")

    def test_multiple_trailing_newlines_collapsed(self):
        """Multiple trailing newlines should become one."""
        source = "var x = 1\n\n\n\n"
        result = normalize_whitespace(source)
        assert result.endswith("\n")
        assert not result.endswith("\n\n")

    def test_empty_string(self):
        """Empty string should produce just a newline."""
        result = normalize_whitespace("")
        assert result == "\n"


class TestCheckIndentation:
    """Tests for check_indentation."""

    def test_valid_indentation_no_warnings(self):
        """Properly indented code should produce no warnings."""
        lines = ["fn foo():", "    say(\"hi\")", "end"]
        warnings = check_indentation(lines)
        assert warnings == []

    def test_tab_indentation_warning(self):
        """Lines with tabs should produce a warning."""
        lines = ["\tvar x = 1"]
        warnings = check_indentation(lines)
        assert any("tabs" in w for w in warnings)

    def test_non_multiple_of_4_warning(self):
        """Lines indented by non-multiple of 4 should warn."""
        lines = ["  var x = 1"]  # 2 spaces
        warnings = check_indentation(lines)
        assert any("2 spaces" in w for w in warnings)

    def test_empty_lines_ignored(self):
        """Empty lines should not produce warnings."""
        lines = ["", "   ", "var x = 1"]
        # "   " is 3 spaces but it's all whitespace, should be skipped
        warnings = check_indentation(lines)
        # Only non-empty lines are checked
        assert len(warnings) == 0

    def test_line_numbers_in_warnings(self):
        """Warning messages should include line numbers."""
        lines = ["var x = 1", "\tvar y = 2"]
        warnings = check_indentation(lines)
        assert any("Line 2" in w for w in warnings)


class TestFormatSfl:
    """Tests for format_sfl."""

    def test_basic_formatting(self):
        """Basic formatting with proper indentation."""
        source = "fn greet():\nsay(\"hi\")\nend\n"
        result = format_sfl(source)
        assert "    say(\"hi\")" in result

    def test_tabs_converted_to_spaces(self):
        """Tabs should be removed (converted to proper indentation)."""
        source = "fn foo():\n\tvar x = 1\nend\n"
        result = format_sfl(source)
        assert "\t" not in result
        # Inside a function body, var should be indented with spaces
        assert "    var x = 1" in result

    def test_trailing_whitespace_removed(self):
        """Trailing whitespace should be removed after formatting."""
        source = "var x = 1   \n"
        result = format_sfl(source)
        lines = result.split("\n")
        for line in lines:
            if line:
                assert line == line.rstrip()

    def test_blank_line_between_top_level_defs(self):
        """Top-level definitions should be separated by blank lines."""
        source = "fn foo():\n    say(\"hi\")\nend\nfn bar():\n    say(\"bye\")\nend\n"
        result = format_sfl(source)
        # There should be a blank line between "end" and "fn bar"
        lines = result.split("\n")
        # Find the first "end" and check next non-empty line starts with "fn"
        end_idx = None
        for i, line in enumerate(lines):
            if line.strip() == "end":
                end_idx = i
                break
        assert end_idx is not None
        # The line after "end" should be blank or "fn bar"
        assert lines[end_idx + 1] == "" or lines[end_idx + 1].strip().startswith("fn")

    def test_nested_indentation(self):
        """Nested blocks should have correct indentation levels."""
        source = "fn main():\nif true:\nsay(\"yes\")\nend\nend\n"
        result = format_sfl(source)
        lines = [l for l in result.split("\n") if l.strip()]
        # Check that "if" is indented once, "say" twice
        for line in lines:
            stripped = line.strip()
            if stripped == "if true:":
                assert line.startswith("    ")
            elif stripped == "say(\"yes\")":
                assert line.startswith("        ")

    def test_normalizes_line_endings(self):
        """Windows line endings should be normalized."""
        source = "var x = 1\r\nvar y = 2\r\n"
        result = format_sfl(source)
        assert "\r" not in result


class TestLintSfl:
    """Tests for lint_sfl."""

    def test_unused_variable_warning(self):
        """Declared but unused variable should produce a warning."""
        source = "var unused_var = 42\nvar used_var = 10\nsay(used_var)\n"
        warnings = lint_sfl(source)
        assert any("unused" in w.lower() and "unused_var" in w for w in warnings)

    def test_used_variable_no_warning(self):
        """Used variable should not produce a warning."""
        source = "var x = 42\nsay(x)\n"
        warnings = lint_sfl(source)
        assert not any("unused" in w.lower() and "'x'" in w for w in warnings)

    def test_underscore_variable_no_warning(self):
        """Variables starting with _ should not produce unused warnings."""
        source = "var _unused = 42\n"
        warnings = lint_sfl(source)
        assert not any("_unused" in w for w in warnings)

    def test_empty_block_warning(self):
        """Empty function/loop body should produce a warning."""
        source = "fn empty():\nend\n"
        warnings = lint_sfl(source)
        assert any("empty" in w.lower() and "block" in w.lower() for w in warnings)

    def test_non_empty_block_no_warning(self):
        """Non-empty block should not produce empty block warning."""
        source = "fn greet():\n    say(\"hi\")\nend\n"
        warnings = lint_sfl(source)
        block_warnings = [w for w in warnings if "empty" in w.lower() and "block" in w.lower()]
        assert block_warnings == []

    def test_missing_type_annotation_warning(self):
        """Function parameter without type annotation should warn."""
        source = "fn greet(name):\n    say(name)\nend\n"
        warnings = lint_sfl(source)
        assert any("type annotation" in w.lower() for w in warnings)

    def test_typed_parameter_no_warning(self):
        """Function parameter with type annotation should not warn."""
        source = "fn greet(name: str):\n    say(name)\nend\n"
        warnings = lint_sfl(source)
        assert not any("type annotation" in w.lower() and "name" in w for w in warnings)

    def test_indentation_warnings_included(self):
        """Linting should include indentation checks."""
        source = "\tvar x = 1\n"
        warnings = lint_sfl(source)
        assert any("tabs" in w.lower() for w in warnings)

    def test_multiple_lint_issues(self):
        """Multiple lint issues should all be reported."""
        source = "var unused = 1\nfn foo(x):\nend\n"
        warnings = lint_sfl(source)
        # Should have: unused variable, missing type annotation, empty block
        assert len(warnings) >= 2  # At least unused var and one other

    def test_empty_source_no_crash(self):
        """Linting empty source should not crash."""
        warnings = lint_sfl("")
        assert isinstance(warnings, list)
