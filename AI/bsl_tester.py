"""
bsl_tester.py - BSL Syntax Validator

Validates .bsl scripts for correct syntax according to the BSL spec.
Can optionally use the buffy --check binary for full validation.

Features:
  - Rule-based validation (works without buffy binary)
  - Optional buffy --check integration (more thorough)
  - Detailed error reporting with line numbers
  - Warning for non-blocking issues (missing metadata, etc.)

Usage:
    from bsl_tester import validate_bsl, ValidationResult
    result = validate_bsl(script_source)
    if result.is_valid:
        print("Script is valid!")
    else:
        for err in result.errors:
            print(f"Line {err.line}: {err.message}")
"""

import re
import subprocess
import tempfile
import os
import shutil
from typing import List, Optional, Tuple


# ── Validation Error Types ─────────────────────────────────────────────────

class ValidationError:
    """A single validation error or warning."""

    def __init__(self, line: int, message: str, severity: str = "error"):
        self.line = line
        self.message = message
        self.severity = severity  # "error" or "warning"

    def __repr__(self):
        prefix = "ERROR" if self.severity == "error" else "WARN"
        return f"[{prefix}] Line {self.line}: {self.message}"


class ValidationResult:
    """Result of validating a .bsl script."""

    def __init__(self):
        self.errors: List[ValidationError] = []
        self.warnings: List[ValidationError] = []
        self.stats: dict = {}

    @property
    def is_valid(self) -> bool:
        return len(self.errors) == 0

    def add_error(self, line: int, message: str):
        err = ValidationError(line, message, "error")
        self.errors.append(err)

    def add_warning(self, line: int, message: str):
        warn = ValidationError(line, message, "warning")
        self.warnings.append(warn)

    def summary(self) -> str:
        parts = []
        if self.errors:
            parts.append(f"{len(self.errors)} error(s)")
        if self.warnings:
            parts.append(f"{len(self.warnings)} warning(s)")
        if not self.errors and not self.warnings:
            return "No issues found."
        return ", ".join(parts) + "."

    def detailed_report(self) -> str:
        lines = []
        for err in self.errors:
            lines.append(f"  [{err.severity.upper()}] Line {err.line}: {err.message}")
        for warn in self.warnings:
            lines.append(f"  [{warn.severity.upper()}] Line {warn.line}: {warn.message}")
        lines.append("")
        lines.append(f"  Summary: {self.summary()}")
        return "\n".join(lines)


# ── Valid Instructions ─────────────────────────────────────────────────────

VALID_INSTRUCTIONS = {"WRITE", "RUN", "WAIT", "CLEAR", "EXIT"}
VALID_METADATA_KEYS = {"VERSION", "AUTHOR", "DESCRIPTION", "OUTPUT"}
BUILTIN_VARIABLES = {"HOME", "USER", "PWD", "TEMP", "DATE", "TIME"}


# ── Line Parser ────────────────────────────────────────────────────────────

def parse_lines(source: str) -> List[Tuple[int, str, str]]:
    """
    Parse source into a list of (line_number, raw_line, content) tuples.
    Strips comments and empty lines.
    """
    result = []
    lines = source.split("\n")
    for i, raw_line in enumerate(lines, 1):
        stripped = raw_line.strip()

        # Skip empty lines
        if not stripped:
            continue

        # Remove inline comments (but not inside quotes)
        content = _strip_comment(stripped)

        # Skip comment-only lines
        if not content:
            continue

        result.append((i, raw_line, content))

    return result


def _strip_comment(line: str) -> str:
    """
    Strip inline comments, being careful not to strip // inside quotes.
    """
    # Find the first // that's NOT inside quotes
    in_quote = False
    quote_char = None
    for i, ch in enumerate(line):
        if ch in ('"', "'"):
            if not in_quote:
                in_quote = True
                quote_char = ch
            elif ch == quote_char:
                # Check for escaped quote
                if i > 0 and line[i - 1] == "\\":
                    continue
                in_quote = False
        elif ch == "/" and i + 1 < len(line) and line[i + 1] == "/":
            if not in_quote:
                return line[:i].strip()
    return line.strip()


# ── Rule-Based Validation ──────────────────────────────────────────────────

def validate_bsl(source: str, use_buffy_check: bool = False) -> ValidationResult:
    """
    Validate a BSL script for correct syntax.

    Args:
        source: The BSL source code as a string.
        use_buffy_check: If True, additionally run buffy --check on the source.

    Returns:
        ValidationResult with errors, warnings, and stats.
    """
    result = ValidationResult()
    parsed = parse_lines(source)

    # Track state
    metadata_section_active = True
    seen_instructions = set()
    seen_metadata = set()
    required_metadata = {"VERSION", "AUTHOR", "DESCRIPTION"}
    write_count = 0
    run_count = 0
    has_exit = False
    output_toggles = 0

    # ── Check each line ──
    for line_num, raw_line, content in parsed:
        # Try to identify the instruction or metadata
        instruction = _identify_instruction(content)

        if instruction is None:
            result.add_warning(line_num, f"Unrecognized statement: {content[:50]}")
            continue

        instr_type = instruction["type"]

        if instr_type == "metadata":
            key = instruction["key"]
            value = instruction["value"]

            if not metadata_section_active:
                result.add_error(line_num, f"Metadata '{key}' found after instructions. All metadata must come first.")
                continue

            seen_metadata.add(key)

            # Validate specific metadata
            if key == "VERSION":
                if not _is_valid_version(value):
                    result.add_warning(line_num, f"VERSION '{value}' doesn't follow YYYY.MM.DD format (recommended).")

            elif key == "OUTPUT":
                if value not in ("true", "false"):
                    result.add_error(line_num, f"OUTPUT must be 'true' or 'false', got '{value}'.")

            elif key == "DESCRIPTION":
                if not value or len(value) < 3:
                    result.add_warning(line_num, "DESCRIPTION is too short (min 3 characters).")

            elif key == "AUTHOR":
                if not value or len(value) < 2:
                    result.add_warning(line_num, "AUTHOR is too short.")

        elif instr_type == "instruction":
            metadata_section_active = False
            instr_name = instruction["name"]
            instr_arg = instruction.get("arg", "")

            seen_instructions.add(instr_name)

            if instr_name == "WRITE":
                write_count += 1
                if not instr_arg:
                    # Empty WRITE "" is allowed (blank line)
                    pass
                elif instr_arg and instr_arg[0] not in ('"', "'"):
                    result.add_warning(line_num, "WRITE argument should be quoted: WRITE \"text\".")

            elif instr_name == "RUN":
                run_count += 1
                if not instr_arg:
                    result.add_error(line_num, "RUN requires a command argument: RUN \"command\".")
                elif instr_arg and instr_arg[0] not in ('"', "'"):
                    result.add_warning(line_num, "RUN argument should be quoted: RUN \"command\".")

                # Check for dangerous commands
                if instr_arg:
                    lower_arg = instr_arg.lower()
                    dangerous = ["rm -rf /", "rm -rf /*", "mkfs", "dd if=", "> /dev/sda"]
                    for danger in dangerous:
                        if danger in lower_arg:
                            result.add_warning(line_num, f"Potentially dangerous command detected: '{instr_arg[:50]}'.")

            elif instr_name == "WAIT":
                if not instr_arg:
                    result.add_error(line_num, "WAIT requires an argument: WAIT <seconds> or WAIT \"message\".")
                else:
                    # Check if it's a number
                    if instr_arg[0] not in ('"', "'"):
                        try:
                            seconds = int(instr_arg)
                            if seconds < 0:
                                result.add_warning(line_num, f"WAIT with negative seconds ({seconds}) has no effect.")
                        except ValueError:
                            result.add_warning(line_num, f"WAIT argument '{instr_arg}' looks like neither a number nor a quoted string.")

            elif instr_name == "OUTPUT":
                metadata_section_active = False
                output_toggles += 1
                if instr_arg not in ("true", "false"):
                    result.add_error(line_num, f"OUTPUT toggle must be 'true' or 'false', got '{instr_arg}'.")

            elif instr_name == "CLEAR":
                metadata_section_active = False
                # CLEAR takes no arguments

            elif instr_name == "EXIT":
                metadata_section_active = False
                has_exit = True

    # ── Variable usage checks ──
    var_errors = _validate_variable_usage(source)
    for ve in var_errors:
        if ve.severity == "error":
            result.errors.append(ve)
        else:
            result.warnings.append(ve)

    # ── Post-validation checks ──

    # Check for missing required metadata
    for meta in required_metadata:
        if meta not in seen_metadata:
            # Only warn if the script has instructions (metadata is optional for trivial scripts)
            if seen_instructions:
                result.add_warning(0, f"Missing required metadata '{meta}'.")

    # Check for EXIT at end
    if seen_instructions and not has_exit and run_count > 0:
        result.add_warning(0, "Script has RUN commands but no EXIT statement at the end.")

    # Check for too many RUN statements
    if run_count > 20:
        result.add_warning(0, f"Script has {run_count} RUN statements (>20). Consider splitting into multiple scripts.")

    # Check for empty script
    if not seen_instructions and not seen_metadata:
        result.add_error(0, "Script is empty.")

    # ── Stats ──
    result.stats = {
        "total_lines": len(source.split("\n")),
        "non_empty_lines": len(parsed),
        "metadata_count": len(seen_metadata),
        "write_count": write_count,
        "run_count": run_count,
        "has_exit": has_exit,
        "output_toggles": output_toggles,
    }

    # ── Optional: buffy --check ──
    if use_buffy_check:
        _run_buffy_check(source, result)

    return result


def _identify_instruction(content: str) -> Optional[dict]:
    """
    Identify what type of instruction a line is.
    Returns a dict with type and parsed data, or None if unrecognized.
    """
    # Check for VERSION = "..." (metadata)
    meta_match = re.match(r'^\s*(VERSION|AUTHOR|DESCRIPTION|OUTPUT)\s*=\s*(.*?)\s*$', content)
    if meta_match:
        key = meta_match.group(1)
        value = meta_match.group(2).strip()
        # Strip quotes from value
        if value.startswith('"') and value.endswith('"'):
            value = value[1:-1]
        elif value.startswith("'") and value.endswith("'"):
            value = value[1:-1]
        return {"type": "metadata", "key": key, "value": value}

    # Check for WRITE, RUN, WAIT, CLEAR, EXIT
    instr_match = re.match(r'^\s*(WRITE|RUN|WAIT|CLEAR|EXIT)\b\s*(.*?)$', content)
    if instr_match:
        name = instr_match.group(1)
        arg = instr_match.group(2).strip()
        # Strip outer quotes from argument
        if arg.startswith('"') and arg.endswith('"'):
            arg = arg[1:-1]
        elif arg.startswith("'") and arg.endswith("'"):
            arg = arg[1:-1]
        return {"type": "instruction", "name": name, "arg": arg}

    # Check for OUTPUT = true/false (runtime toggle)
    output_match = re.match(r'^\s*OUTPUT\s*=\s*(true|false)\s*$', content)
    if output_match:
        return {"type": "instruction", "name": "OUTPUT", "arg": output_match.group(1)}

    return None


def _is_valid_version(version: str) -> bool:
    """Check if a version string follows YYYY.MM.DD format."""
    patterns = [
        r'^\d{4}\.\d{2}\.\d{2}$',           # YYYY.MM.DD
        r'^\d{4}\.\d{2}\.\d{2}\.\d+$',      # YYYY.MM.DD.N
        r'^\d+\.\d+\.\d+$',                 # Semver
    ]
    for pattern in patterns:
        if re.match(pattern, version):
            return True
    return False


def _validate_variable_usage(source: str) -> List[ValidationError]:
    """Check that variable references use correct syntax."""
    errors = []
    lines = source.split("\n")
    var_pattern = re.compile(r'\$\{(\w+)\}')

    for i, line in enumerate(lines, 1):
        # Skip comments
        if line.strip().startswith("//"):
            continue

        matches = var_pattern.findall(line)
        for var_name in matches:
            # Variables that start with a digit are argument references (${1}, ${2})
            if var_name.isdigit():
                continue
            # Check if it's a known variable
            if var_name not in BUILTIN_VARIABLES:
                errors.append(ValidationError(
                    i,
                    f"Unknown variable '${{{var_name}}}'. Built-in variables: {', '.join(sorted(BUILTIN_VARIABLES))}",
                    "warning"
                ))

    return errors


def _run_buffy_check(source: str, result: ValidationResult):
    """
    Run the buffy --check command on the source to validate syntax.
    Falls back silently if buffy is not available.
    """
    if not shutil.which("buffy"):
        result.add_warning(0, "Skipping buffy --check: 'buffy' command not found in PATH.")
        return

    # Write source to a temp file and run buffy --check
    with tempfile.NamedTemporaryFile(mode="w", suffix=".bsl", delete=False) as f:
        f.write(source)
        temp_path = f.name

    try:
        proc = subprocess.run(
            ["buffy", "--check", temp_path],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if proc.returncode != 0:
            stderr = proc.stderr.strip()
            stdout = proc.stdout.strip()
            # Parse buffy's error output for useful info
            error_text = stderr if stderr else stdout
            if error_text:
                result.add_error(0, f"buffy --check: {error_text[:200]}")
            else:
                result.add_error(0, "buffy --check reported an error (no details).")
    except subprocess.TimeoutExpired:
        result.add_warning(0, "buffy --check timed out.")
    except Exception as e:
        result.add_warning(0, f"buffy --check failed: {e}")
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)


# ── Export Validation ──────────────────────────────────────────────────────

def format_as_bsl(source: str) -> str:
    """
    Clean up and format a BSL script to match canonical style.
    Ensures proper spacing, trailing newline, and consistent formatting.
    """
    lines = source.split("\n")

    # Remove leading/trailing blank lines
    while lines and lines[0].strip() == "":
        lines.pop(0)
    while lines and lines[-1].strip() == "":
        lines.pop()

    # Ensure EXIT at the end
    if lines and lines[-1].strip() != "EXIT":
        lines.append("")
        lines.append("EXIT")

    formatted = "\n".join(lines) + "\n"
    return formatted


if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1 and sys.argv[1] == "--check":
        # Read from stdin or file
        if len(sys.argv) > 2:
            with open(sys.argv[2]) as f:
                source = f.read()
        else:
            source = sys.stdin.read()

        result = validate_bsl(source)
        print(result.detailed_report())
        print(f"Stats: {result.stats}")
        sys.exit(0 if result.is_valid else 1)

    elif len(sys.argv) > 1:
        # Validate a file
        with open(sys.argv[1]) as f:
            source = f.read()
        result = validate_bsl(source)
        print(result.detailed_report())

    else:
        # Run self-test
        test_source = '''VERSION = "2026.07.28"
AUTHOR = "Buffy Community"
DESCRIPTION = "Test script for validation."
OUTPUT = false

WRITE "Hello, World!"
RUN "echo test"
WAIT 1
EXIT
'''
        result = validate_bsl(test_source)
        print(f"Self-test: {'PASSED' if result.is_valid else 'FAILED'}")
        if not result.is_valid:
            print(result.detailed_report())
