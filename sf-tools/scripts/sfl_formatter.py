"""SFL file formatter and linter for Sailfish Studio.

Provides formatting (indentation, spacing) and linting (unused variables,
empty blocks, missing type annotations) for .sfl source files.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field


# Keywords that introduce a new block (increase indentation)
_BLOCK_KEYWORDS = {"fn", "if", "else", "while", "repeat", "for", "match"}

# Keywords that end a block or start at same level
_BLOCK_END = {"end", "else", "elif"}

# Top-level definition keywords
_TOP_LEVEL_KEYWORDS = {"fn", "var", "const", "type"}


def normalize_whitespace(source: str) -> str:
    """Normalize line endings and trailing whitespace.

    - Converts Windows line endings (\\r\\n) to Unix (\\n)
    - Removes trailing whitespace from every line
    - Ensures file ends with a single newline

    Args:
        source: The .sfl source code string.

    Returns:
        Normalized source string.
    """
    # Normalize line endings
    source = source.replace("\r\n", "\n").replace("\r", "\n")

    # Remove trailing whitespace on each line
    lines = source.split("\n")
    lines = [line.rstrip() for line in lines]

    # Remove trailing empty lines
    while lines and lines[-1] == "":
        lines.pop()

    # Ensure file ends with a newline
    return "\n".join(lines) + "\n"


def check_indentation(lines: list[str]) -> list[str]:
    """Check that indentation is consistent (4-space indentation, no tabs).

    Args:
        lines: List of source code lines (without line endings).

    Returns:
        List of warning messages for indentation issues.
    """
    warnings: list[str] = []
    for i, line in enumerate(lines, start=1):
        if not line or line.isspace():
            continue

        # Check for tabs
        if "\t" in line:
            warnings.append(f"Line {i}: uses tabs instead of spaces")

        # Check indentation level
        stripped = line.lstrip()
        indent_len = len(line) - len(stripped)
        if indent_len > 0 and indent_len % 4 != 0:
            warnings.append(
                f"Line {i}: indentation of {indent_len} spaces is not a multiple of 4"
            )

    return warnings


def format_sfl(source: str) -> str:
    """Format .sfl source code with consistent style.

    Applies the following formatting rules:
    - 4-space indentation (no tabs)
    - No trailing whitespace
    - Single blank line between top-level definitions
    - Normalized line endings

    Args:
        source: The .sfl source code string.

    Returns:
        Formatted source string.
    """
    # First normalize
    source = normalize_whitespace(source)

    lines = source.split("\n")
    # Remove the trailing empty element from split (since normalize adds \n)
    if lines and lines[-1] == "":
        lines = lines[:-1]

    # Convert tabs to spaces
    formatted_lines: list[str] = []
    for line in lines:
        # Replace leading tabs with 4-space groups
        if line.startswith("\t"):
            stripped = line.lstrip("\t")
            tab_count = len(line) - len(stripped)
            line = "    " * tab_count + stripped
        formatted_lines.append(line)

    # Re-indent based on block structure
    result_lines: list[str] = []
    indent_level = 0
    prev_was_top_level = False

    for line in formatted_lines:
        stripped = line.strip()
        if not stripped:
            result_lines.append("")
            continue

        # Skip comment-only lines
        if stripped.startswith("//"):
            result_lines.append("    " * indent_level + stripped)
            continue

        # Decrease indent for block-end keywords
        if stripped.startswith("end") and stripped in ("end", "end "):
            indent_level = max(0, indent_level - 1)
        elif stripped.startswith("else") or stripped.startswith("elif"):
            indent_level = max(0, indent_level - 1)

        # Determine if this is a top-level definition
        first_word = stripped.split()[0] if stripped.split() else ""
        is_top_level = first_word in _TOP_LEVEL_KEYWORDS and indent_level == 0

        # Add blank line between top-level definitions
        if is_top_level and prev_was_top_level and result_lines:
            # Ensure blank line before
            if result_lines and result_lines[-1] != "":
                result_lines.append("")

        # Apply indentation
        new_line = "    " * indent_level + stripped
        result_lines.append(new_line)

        # Increase indent after block-start keywords
        if any(stripped.startswith(kw) for kw in _BLOCK_KEYWORDS):
            if not stripped.endswith("end"):
                indent_level += 1

        # After else/elif, increase indent
        if stripped.startswith("else") or stripped.startswith("elif"):
            indent_level += 1

        # After end, indent already decreased
        if stripped.startswith("end"):
            pass  # Already handled above

        prev_was_top_level = is_top_level

    # Remove multiple consecutive blank lines
    final_lines: list[str] = []
    blank_count = 0
    for line in result_lines:
        if line == "":
            blank_count += 1
            if blank_count <= 1:
                final_lines.append(line)
        else:
            blank_count = 0
            final_lines.append(line)

    # Remove trailing blank lines and ensure final newline
    while final_lines and final_lines[-1] == "":
        final_lines.pop()

    return "\n".join(final_lines) + "\n"


def lint_sfl(source: str) -> list[str]:
    """Lint .sfl source code and return a list of warnings.

    Checks for:
    - Unused variables (variables starting with _ are excluded)
    - Empty blocks (e.g., fn foo() end with nothing inside)
    - Missing type annotations on function parameters

    Args:
        source: The .sfl source code string.

    Returns:
        List of lint warning messages.
    """
    warnings: list[str] = []

    lines = source.split("\n")

    # Check indentation
    warnings.extend(check_indentation(lines))

    # Collect variable declarations and usages
    declared_vars: dict[str, int] = {}  # name -> line number
    used_vars: set[str] = set()

    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("//"):
            continue

        # Detect var declarations: var name = value
        var_match = re.match(r"var\s+(\w+)", stripped)
        if var_match:
            var_name = var_match.group(1)
            if not var_name.startswith("_"):
                declared_vars[var_name] = i

        # Detect variable usage (simple heuristic: word boundaries)
        for var_name in declared_vars:
            # Check if var_name appears as a standalone identifier
            pattern = r"\b" + re.escape(var_name) + r"\b"
            # Don't count the declaration line as usage
            if re.search(pattern, stripped) and not re.match(
                rf"var\s+{re.escape(var_name)}", stripped
            ):
                used_vars.add(var_name)

    # Report unused variables
    for var_name, line_num in declared_vars.items():
        if var_name not in used_vars:
            warnings.append(f"Line {line_num}: unused variable '{var_name}'")

    # Check for empty blocks
    in_block = False
    block_start_line = 0
    block_start_type = ""
    block_content_found = False
    block_depth = 0

    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("//"):
            continue

        first_word = stripped.split()[0] if stripped.split() else ""

        if first_word in _BLOCK_KEYWORDS:
            if not in_block:
                in_block = True
                block_start_line = i
                block_start_type = first_word
                block_content_found = False
                block_depth = 1
            else:
                block_depth += 1
                block_content_found = True  # Nested block counts as content
        elif stripped == "end":
            if in_block:
                block_depth -= 1
                if block_depth == 0:
                    if not block_content_found:
                        warnings.append(
                            f"Line {block_start_line}: empty {block_start_type} block"
                        )
                    in_block = False
        elif in_block:
            block_content_found = True

    # Check for missing type annotations on function parameters
    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        # Match fn declarations with parameters
        fn_match = re.match(r"fn\s+(\w+)\s*\(([^)]*)\)", stripped)
        if fn_match:
            params_str = fn_match.group(2).strip()
            if params_str:
                params = [p.strip() for p in params_str.split(",")]
                for param in params:
                    # A parameter with type annotation looks like: name: Type
                    # Without annotation: just name
                    if ":" not in param and param != "":
                        param_name = param.strip()
                        if param_name and not param_name.startswith("//"):
                            warnings.append(
                                f"Line {i}: parameter '{param_name}' missing type annotation"
                            )

    return warnings
