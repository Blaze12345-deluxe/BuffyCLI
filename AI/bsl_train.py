"""
bsl_train.py - BSL Training Pipeline

Loads the training corpus, analyzes patterns, and prepares the
generator for use. This is the "training" phase — it loads data,
runs diagnostics on the corpus, and can optionally export the
training data for use by external AI/LLM systems.

Usage:
    python bsl_train.py                  # Show training summary
    python bsl_train.py --export         # Export training data as JSON
    python bsl_train.py --export-format  # Export BSL format reference
"""

import json
import sys
import os
from datetime import datetime
from typing import List, Dict

from training_data import (
    TRAINING_EXAMPLES,
    PATTERN_TEMPLATES,
    INSTRUCTION_PATTERNS,
    get_all_tags,
    get_all_dependencies,
    summary as corpus_summary,
)


class BSLAITrainer:
    """
    Trains/loads the BSL generation system.
    In this implementation, "training" means loading the corpus
    and preparing data structures for the generator.
    """

    def __init__(self):
        self.corpus = TRAINING_EXAMPLES
        self.templates = PATTERN_TEMPLATES
        self.instructions = INSTRUCTION_PATTERNS
        self.tags = get_all_tags()
        self.dependencies = get_all_dependencies()
        self.trained_at = datetime.now().isoformat()

    def train_summary(self) -> Dict:
        """Return a detailed summary of the training state."""
        return {
            "name": "BSL AI Trainer",
            "version": "1.0.0",
            "trained_at": self.trained_at,
            "corpus": {
                "total_examples": len(self.corpus),
                "total_tags": len(self.tags),
                "total_dependencies": len(self.dependencies),
                "tags": self.tags,
                "dependencies": self.dependencies,
                "script_names": [ex["name"] for ex in self.corpus],
            },
            "templates": {
                "count": len(self.templates),
                "names": list(self.templates.keys()),
            },
            "instructions": {
                "count": len(self.instructions),
                "names": list(self.instructions.keys()),
            },
        }

    def export_corpus_json(self, filepath: Optional[str] = None) -> str:
        """
        Export the training corpus as JSON for use by external AI/LLM systems.
        If filepath is provided, writes to file. Returns the JSON string.
        """
        export = {
            "metadata": {
                "format": "bsl-training-corpus",
                "version": "1.0.0",
                "exported_at": self.trained_at,
                "total_examples": len(self.corpus),
                "description": "BSL (Buffy Script Language) training examples for AI script generation",
            },
            "language_spec": {
                "file_extension": ".bsl",
                "encoding": "UTF-8",
                "metadata_fields": ["VERSION", "AUTHOR", "DESCRIPTION", "OUTPUT"],
                "instructions": [
                    {
                        "name": name,
                        "syntax": info["pattern"],
                        "description": info["description"],
                        "notes": info["usage_notes"],
                    }
                    for name, info in self.instructions.items()
                ],
                "builtin_variables": ["HOME", "USER", "PWD", "TEMP", "DATE", "TIME"],
            },
            "templates": {
                name: tmpl["structure"]
                for name, tmpl in self.templates.items()
            },
            "examples": [
                {
                    "name": ex["name"],
                    "description": ex["description"],
                    "tags": ex["tags"],
                    "dependencies": ex["dependencies"],
                    "args": ex["args"],
                    "source": ex["source"],
                }
                for ex in self.corpus
            ],
        }

        json_str = json.dumps(export, indent=2)

        if filepath:
            with open(filepath, "w") as f:
                f.write(json_str)
            print(f"Corpus exported to {filepath} ({len(json_str)} bytes)")

        return json_str

    def export_format_reference(self) -> str:
        """Export a human-readable BSL format reference."""
        lines = []
        lines.append("=" * 60)
        lines.append("  BSL (Buffy Script Language) Format Reference")
        lines.append("=" * 60)
        lines.append("")

        # Metadata
        lines.append("METADATA (must appear before any instructions):")
        lines.append("-" * 40)
        lines.append('  VERSION     = "YYYY.MM.DD"')
        lines.append('  AUTHOR      = "Creator name"')
        lines.append('  DESCRIPTION = "What the script does"')
        lines.append("  OUTPUT      = true  (visible) or false (silent)")
        lines.append("")

        # Instructions
        lines.append("INSTRUCTIONS (one per line, executed top-to-bottom):")
        lines.append("-" * 40)
        for name, info in self.instructions.items():
            lines.append(f"  {info['pattern']}")
            lines.append(f"      {info['description']}")
        lines.append("")

        # Variables
        lines.append("BUILT-IN VARIABLES (expanded at runtime):")
        lines.append("-" * 40)
        lines.append("  ${HOME}    - Home directory")
        lines.append("  ${USER}    - Current username")
        lines.append("  ${PWD}     - Current working directory")
        lines.append("  ${TEMP}    - System temp directory")
        lines.append("  ${DATE}    - Current date (YYYY-MM-DD)")
        lines.append("  ${TIME}    - Current time (HH:MM:SS)")
        lines.append("  ${1}-${N}  - Script arguments")
        lines.append("")

        # Comments
        lines.append("COMMENTS:")
        lines.append("-" * 40)
        lines.append("  // This is a comment")
        lines.append('  WRITE "hi"  // Inline comment')
        lines.append("")

        # Examples summary
        lines.append(f"CORPUS: {len(self.corpus)} training examples")
        lines.append("-" * 40)
        for ex in self.corpus:
            args_str = ", ".join(ex["args"]) if ex["args"] else "(none)"
            deps_str = ", ".join(ex["dependencies"]) if ex["dependencies"] else "(none)"
            lines.append(f"  {ex['name']}")
            lines.append(f"    Description: {ex['description']}")
            lines.append(f"    Args: {args_str}")
            lines.append(f"    Deps: {deps_str}")
        lines.append("")

        return "\n".join(lines)


def main():
    trainer = BSLAITrainer()

    if len(sys.argv) > 1:
        if sys.argv[1] == "--export":
            filepath = sys.argv[2] if len(sys.argv) > 2 else None
            trainer.export_corpus_json(filepath)

        elif sys.argv[1] == "--export-format":
            print(trainer.export_format_reference())

        elif sys.argv[1] == "--export-all":
            # Export to a known location
            output_dir = sys.argv[2] if len(sys.argv) > 2 else "."
            os.makedirs(output_dir, exist_ok=True)
            corpus_path = os.path.join(output_dir, "bsl-corpus.json")
            fmt_path = os.path.join(output_dir, "bsl-format-reference.txt")
            trainer.export_corpus_json(corpus_path)
            with open(fmt_path, "w") as f:
                f.write(trainer.export_format_reference())
            print(f"Exported to {output_dir}/")

        else:
            print(f"Unknown option: {sys.argv[1]}")
            print("Usage: python bsl_train.py [--export [file] | --export-format | --export-all [dir]]")
    else:
        # Default: show summary
        summary = trainer.train_summary()
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
