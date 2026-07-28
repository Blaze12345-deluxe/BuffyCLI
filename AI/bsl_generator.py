"""
bsl_generator.py - BSL Script Generator

Takes a natural language prompt and generates a valid .bsl script.
Uses pattern matching, template assembly, and the training corpus
to create scripts that match the user's request.

Usage:
    from bsl_generator import generate_bsl
    script = generate_bsl("create a script that monitors disk space")
"""

import re
import textwrap
from typing import List, Optional

from datetime import datetime

from training_data import (
    TRAINING_EXAMPLES,
    PATTERN_TEMPLATES,
)


# ── BSL Syntax Templates ───────────────────────────────────────────────────

METADATA_TEMPLATE = 'VERSION = "{version}"\n' \
                    'AUTHOR = "{author}"\n' \
                    'DESCRIPTION = "{description}"\n' \
                    'OUTPUT = {output}\n'

CLEAR_STATEMENT = "CLEAR"
EXIT_STATEMENT = "EXIT"
WRITE_TEMPLATE = 'WRITE "{text}"'
RUN_TEMPLATE = 'RUN "{command}"'
WAIT_SECONDS_TEMPLATE = "WAIT {seconds}"
WAIT_PROMPT_TEMPLATE = 'WAIT "{prompt}"'
OUTPUT_ON = "OUTPUT = true"
OUTPUT_OFF = "OUTPUT = false"

SECTION_HEADER = 'WRITE "========================================="'
SECTION_TITLE = 'WRITE "  {title}"'
SECTION_DIVIDER = 'WRITE "-----------------------------------------"'
BLANK_LINE = 'WRITE ""'


# ── Scoring and Matching ───────────────────────────────────────────────────

def score_prompt_against_example(prompt: str, example: dict) -> int:
    """
    Score how well a prompt matches a training example.
    Higher score = better match.
    """
    prompt_lower = prompt.lower()
    score = 0

    # Check prompt_keywords
    for keyword in example["prompt_keywords"]:
        if keyword.lower() in prompt_lower:
            score += 10
        # Partial keyword match
        words = keyword.lower().split()
        if any(w in prompt_lower for w in words):
            score += 3

    # Check description
    desc_words = example["description"].lower().split()
    for word in desc_words:
        if len(word) > 3 and word in prompt_lower:
            score += 2

    # Check name
    name = example["name"].lower().replace("-", " ")
    if name in prompt_lower:
        score += 15

    # Check tags
    for tag in example["tags"]:
        if tag.lower() in prompt_lower:
            score += 5

    return score


def detect_required_args(prompt: str, example: dict) -> List[str]:
    """
    Detect if the user specified argument values in the prompt.
    Returns the list of argument values (or empty if none detected).
    """
    # Look for patterns like "for <name>", "called <name>", "<name> directory"
    match = re.search(r'(?:for|called|named)\s+[\'"]?(\w+)[\'"]?', prompt, re.IGNORECASE)
    if match:
        return [match.group(1)]
    return []


def detect_shell_commands(prompt: str) -> List[str]:
    """Extract likely shell commands mentioned in the prompt."""
    known_commands = [
        "docker", "git", "python", "pip", "npm", "cargo", "apt", "brew",
        "tar", "gzip", "curl", "wget", "ssh", "scp", "rsync", "mkdir",
        "rm", "cp", "mv", "ls", "find", "grep", "sed", "awk", "chmod",
        "systemctl", "service", "journalctl", "ping", "nslookup", "df",
        "du", "free", "uname", "ps", "top", "htop", "kill", "screen",
        "tmux", "make", "cmake", "docker-compose", "kubectl",
    ]
    prompt_lower = prompt.lower()
    found = []
    for cmd in known_commands:
        if cmd in prompt_lower:
            found.append(cmd)
    return found


def detect_output_preference(prompt: str) -> bool:
    """
    Detect whether the user wants output visible.
    Returns True for silent (OUTPUT = false), which is the default.
    """
    prompt_lower = prompt.lower()
    silent_indicators = ["silent", "quiet", "background", "automatically", "no output"]
    visible_indicators = ["show", "display", "progress", "verbose", "output"]
    for word in silent_indicators:
        if word in prompt_lower:
            return False
    for word in visible_indicators:
        if word in prompt_lower:
            return True
    return False  # Default to silent


# ── Script Generation ──────────────────────────────────────────────────────

def generate_bsl(prompt: str, author: str = "Buffy Community") -> dict:
    """
    Generate a .bsl script from a natural language prompt.

    Returns a dict with:
      - name: suggested file name
      - source: the complete .bsl source code
      - description: auto-generated description
      - args: list of expected arguments
      - dependencies: list of required shell commands
      - matched_example: name of the best-matching training example
      - match_score: confidence score (higher = better match)
    """
    # Find the best-matching training example
    best_example = None
    best_score = 0

    for example in TRAINING_EXAMPLES:
        score = score_prompt_against_example(prompt, example)
        if score > best_score:
            best_score = score
            best_example = example

    # Analyze the prompt for context
    args = detect_required_args(prompt, best_example) if best_example else []
    shell_commands = detect_shell_commands(prompt)
    output_visible = detect_output_preference(prompt)

    # Generate description from prompt
    description = _generate_description(prompt)

    # Generate name from prompt
    name = _generate_name(prompt)

    # If we have a good match, adapt the example
    if best_example and best_score >= 10:
        source = _adapt_example(
            prompt=prompt,
            example=best_example,
            name=name,
            description=description,
            author=author,
            args=args,
            output_visible=output_visible,
        )
    else:
        # No good match — generate from scratch using patterns
        source = _generate_from_scratch(
            prompt=prompt,
            name=name,
            description=description,
            author=author,
            shell_commands=shell_commands,
            args=args,
            output_visible=output_visible,
        )

    # Determine dependencies
    dependencies = []
    if best_example:
        dependencies = best_example["dependencies"]
    dependencies.extend(shell_commands)
    dependencies = list(set(dependencies))

    return {
        "name": f"{name}.bsl",
        "source": source.strip() + "\n",
        "description": description,
        "args": args,
        "dependencies": dependencies,
        "matched_example": best_example["name"] if best_example else None,
        "match_score": best_score,
    }


def _generate_description(prompt: str) -> str:
    """Generate a concise description from the prompt."""
    # Clean and truncate
    description = prompt.strip()
    description = re.sub(r'\s+', ' ', description)
    if len(description) > 80:
        description = description[:77] + "..."
    # Capitalize first letter
    if description:
        description = description[0].upper() + description[1:]
    return description


def _generate_name(prompt: str) -> str:
    """Generate a kebab-case filename from the prompt."""
    # Extract key action words
    words = re.findall(r'\b(\w+)\b', prompt.lower())
    # Filter out common stop words
    stop_words = {"a", "an", "the", "for", "to", "in", "of", "on", "at", "by",
                  "with", "from", "up", "that", "this", "is", "it", "be", "and",
                  "or", "but", "not", "script", "that", "will", "can", "all",
                  "your", "its", "my", "do", "does", "has", "have", "are", "was"}
    filtered = [w for w in words if w not in stop_words and len(w) > 2]

    if len(filtered) >= 2:
        name = "-".join(filtered[:4])
    elif filtered:
        name = filtered[0]
    else:
        name = "my-script"

    # Remove trailing hyphens
    name = name.strip("-")

    # Avoid duplicate names with existing examples
    existing_names = [ex["name"] for ex in TRAINING_EXAMPLES]
    if name in existing_names:
        name = name + "-custom"

    return name


def _adapt_example(prompt: str, example: dict, name: str, description: str,
                   author: str, args: List[str], output_visible: bool) -> str:
    """
    Take a matching training example and adapt it to the user's prompt.
    Updates metadata, argument handling, and adds relevant comments.
    """
    source = example["source"]

    # Update DESCRIPTION
    source = re.sub(
        r'DESCRIPTION = ".*?"',
        f'DESCRIPTION = "{description}"',
        source,
        count=1,
    )

    # Update AUTHOR
    source = re.sub(
        r'AUTHOR = ".*?"',
        f'AUTHOR = "{author}"',
        source,
        count=1,
    )

    # Update OUTPUT preference
    if not output_visible:
        source = re.sub(
            r'OUTPUT = true\n',
            'OUTPUT = false\n',
            source,
            count=1,
        )

    # Add argument comments if the example expects args
    if example["args"] and not args:
        arg_comment = f"// Usage: buffy {name} <{' '.join(example['args'])}>\n"
        # Insert after the metadata section
        source = re.sub(
            r'(OUTPUT = (?:true|false)\n)',
            r'\1' + '\n' + arg_comment,
            source,
            count=1,
        )

    return source


def _generate_from_scratch(prompt: str, name: str, description: str,
                           author: str, shell_commands: List[str],
                           args: List[str], output_visible: bool) -> str:
    """
    Generate a BSL script from scratch when no good example match exists.
    Uses templates and patterns to build a valid script.
    """
    lines = []

    # ── Comments ──
    lines.append(f"// BSL Script: {name}")
    lines.append(f"// Generated from prompt: {prompt[:60]}")
    lines.append("")

    # ── Metadata ──
    today = datetime.now().strftime("%Y.%m.%d")
    output_mode = "true" if output_visible else "false"
    lines.append(f'VERSION = "{today}"')
    lines.append(f'AUTHOR = "{author}"')
    lines.append(f'DESCRIPTION = "{description}"')
    lines.append(f"OUTPUT = {output_mode}")
    lines.append("")

    # ── Usage comment ──
    if args:
        arg_str = " ".join([f"<{a}>" for a in args])
    else:
        arg_str = ""
    lines.append(f"// Usage: buffy {name} {arg_str}")
    lines.append("")

    # ── Header ──
    nice_name = name.replace("-", " ").title()
    lines.append(SECTION_HEADER)
    lines.append(SECTION_TITLE.format(title=nice_name))
    lines.append(SECTION_HEADER)
    lines.append(BLANK_LINE)

    # ── Arguments display ──
    if args:
        lines.append(SECTION_DIVIDER)
        for i, arg_name in enumerate(args, 1):
            lines.append(f'WRITE "{arg_name}: ${{{i}}}"')
        lines.append(SECTION_DIVIDER)
        lines.append(BLANK_LINE)

    # ── Command execution ──
    for cmd in shell_commands:
        lines.append(f'WRITE "Running {cmd}..."')
        if cmd == "python" or cmd == "python3":
            lines.append(f'RUN "{cmd} --version"')
        elif cmd == "docker":
            lines.append("RUN \"docker info > /dev/null 2>&1 && echo 'Docker is running' || echo 'Docker is not running'\"")
        elif cmd == "git":
            lines.append(f'RUN "{cmd} --version"')
        elif cmd == "apt":
            lines.append(f'RUN "sudo {cmd} update"')
        elif cmd == "df":
            lines.append(f'RUN "{cmd} -h"')
        elif cmd == "du":
            lines.append(f'RUN "{cmd} -sh ."')
        elif cmd == "free":
            lines.append(f'RUN "{cmd} -h | head -3"')
        elif cmd == "uname":
            lines.append(f'RUN "{cmd} -a"')
        elif cmd == "ping":
            lines.append(f'RUN "{cmd} -c 4 ${{1:-localhost}}"')
        elif cmd == "curl":
            if args:
                lines.append(f'RUN "{cmd} -sI ${{1}}"')
            else:
                lines.append(f'RUN "{cmd} --version"')
        elif cmd == "tar":
            if len(args) >= 2:
                lines.append("RUN \"tar -czf '${2}' '${1}'\"")
            else:
                lines.append(f'RUN "{cmd} --version"')
        elif cmd == "mkdir":
            if args:
                lines.append(f'RUN "mkdir -p ${{1}}"')
            else:
                lines.append(f'RUN "mkdir -p ${{1:-new-project}}"')
        elif cmd in ("rm", "cp", "mv", "ls", "find", "grep", "chmod"):
            if cmd == "find":
                lines.append(f'RUN "{cmd} . -type f | head -10"')
            elif cmd == "ls":
                lines.append(f'RUN "{cmd} -la"')
            else:
                lines.append(f'RUN "{cmd}"')
        else:
            lines.append(f'RUN "{cmd}"')
        lines.append("")

    # ── Footer ──
    lines.append(SECTION_HEADER)
    lines.append(f'WRITE "  {nice_name} Complete!"')
    lines.append(SECTION_HEADER)
    lines.append("")

    # ── EXIT ──
    lines.append(EXIT_STATEMENT)

    return "\n".join(lines)


# ── Summary / Analysis ─────────────────────────────────────────────────────

def find_best_match(prompt: str) -> dict:
    """Return the best-matching training example and its score."""
    best = None
    best_score = 0
    for example in TRAINING_EXAMPLES:
        score = score_prompt_against_example(prompt, example)
        if score > best_score:
            best_score = score
            best = example
    return {
        "matched_example": best["name"] if best else None,
        "match_score": best_score,
        "description": best["description"] if best else None,
    }


if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        prompt = " ".join(sys.argv[1:])
    else:
        prompt = "show me disk usage information"
    result = generate_bsl(prompt)
    print(f"Name: {result['name']}")
    print(f"Description: {result['description']}")
    print(f"Matched: {result['matched_example']} (score: {result['match_score']})")
    print(f"Dependencies: {', '.join(result['dependencies'])}")
    print(f"Args: {result['args']}")
    print(f"\n{'-'*60}\n")
    print(result['source'])
