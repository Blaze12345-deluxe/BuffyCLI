#!/usr/bin/env python3
"""
run.py - BSL AI Script Generator CLI

Main entry point for the AI module. Takes a natural language prompt
and generates a complete .bsl script with syntax validation.

Usage:
    python AI/run.py "create a script that backs up my home directory"
    python AI/run.py "show system information" --output system-info.bsl
    python AI/run.py "monitor disk space" --check
    python AI/run.py --train          # Show training corpus summary
    python AI/run.py --interactive    # Interactive mode
    python AI/run.py --help           # Show this help
"""

import sys
import os
import argparse
import textwrap

from bsl_generator import generate_bsl, find_best_match
from bsl_tester import validate_bsl, format_as_bsl
from bsl_train import BSLAITrainer


def main():
    parser = argparse.ArgumentParser(
        prog="buffy-ai",
        description="Generate BSL scripts from natural language prompts",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=textwrap.dedent("""\
            Examples:
              python AI/run.py "create a python virtual environment"
              python AI/run.py "backup my project directory" --output backup.bsl
              python AI/run.py "check disk usage" --check
              python AI/run.py --interactive
              python AI/run.py --train
        """),
    )

    # Main arguments
    parser.add_argument(
        "prompt",
        nargs="*",
        help="Natural language description of the script to generate",
    )
    parser.add_argument(
        "-o", "--output",
        help="Output file path for the generated .bsl script",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Run syntax validation on the generated script",
    )
    parser.add_argument(
        "--buffy-check",
        action="store_true",
        help="Also run buffy --check for full validation (requires buffy in PATH)",
    )
    parser.add_argument(
        "--author",
        default="AI Generated",
        help="Author name for the script (default: AI Generated)",
    )
    parser.add_argument(
        "--interactive",
        action="store_true",
        help="Start interactive mode",
    )
    parser.add_argument(
        "--train",
        action="store_true",
        help="Show training corpus summary",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Only output the generated script (no diagnostics)",
    )

    args = parser.parse_args()

    # ── Show training summary ──
    if args.train:
        if not args.quiet:
            trainer = BSLAITrainer()
            summary = trainer.train_summary()
            print(f"BSL AI Trainer v{summary['version']}")
            print(f"Trained at: {summary['trained_at']}")
            print(f"Training examples: {summary['corpus']['total_examples']}")
            print(f"Tags: {len(summary['corpus']['tags'])}")
            print(f"Template patterns: {summary['templates']['count']}")
            print(f"Known instructions: {summary['instructions']['count']}")
            print(f"Dependencies tracked: {summary['corpus']['total_dependencies']}")
            print(f"\nAvailable tags: {', '.join(summary['corpus']['tags'])}")
        return

    # ── Interactive mode ──
    if args.interactive:
        interactive_loop(args)
        return

    # ── Generate from prompt ──
    if not args.prompt:
        parser.print_help()
        print("\nError: No prompt provided. Describe what script you want to generate.")
        sys.exit(1)

    prompt = " ".join(args.prompt)
    result = generate_bsl(prompt, author=args.author)

    # ── Optional: Validate ──
    validation = None
    if args.check or args.buffy_check:
        validation = validate_bsl(
            result["source"],
            use_buffy_check=args.buffy_check,
        )

    # ── Output ──
    if args.quiet:
        # Only output the source code
        sys.stdout.write(result["source"])
    else:
        # Full diagnostic output
        _print_generation_report(result, validation)

        # Write to file if requested
        if args.output:
            _write_to_file(args.output, result["source"])
        else:
            # Print the source with a header
            print(f"\n{'='*60}")
            print(f"  Generated Script: {result['name']}")
            print(f"{'='*60}")
            print(result["source"])

    # Write to default filename if no output specified
    if not args.output and not args.quiet:
        save = input(f"\nSave to {result['name']}? [Y/n]: ").strip().lower()
        if save != "n":
            _write_to_file(result["name"], result["source"])


def interactive_loop(args):
    """Run an interactive prompt loop."""
    print("=" * 60)
    print("  BSL AI Generator - Interactive Mode")
    print("=" * 60)
    print('  Type a description of the script you want to generate.')
    print('  Type "quit" to exit, "help" for commands.')
    print("=" * 60)
    print()

    while True:
        try:
            prompt = input(">> ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\nGoodbye!")
            break

        if not prompt:
            continue

        if prompt.lower() in ("quit", "exit", "q"):
            print("Goodbye!")
            break

        if prompt.lower() in ("help", "h", "?"):
            _print_interactive_help()
            continue

        if prompt.lower() == "train":
            trainer = BSLAITrainer()
            summary = trainer.train_summary()
            print(f"\nCorpus: {summary['corpus']['total_examples']} examples, "
                  f"{summary['corpus']['total_tags']} tags, "
                  f"{summary['templates']['count']} templates\n")
            continue

        # Generate
        result = generate_bsl(prompt, author=args.author)
        validation = validate_bsl(result["source"])

        if validation.is_valid:
            print(f"\n  Generated: {result['name']} "
                  f"(matched: {result['matched_example']}, "
                  f"score: {result['match_score']})\n")
        else:
            print(f"\n  Generated: {result['name']} "
                  f"({validation.summary()})\n")

        # Show the script
        print(result["source"])
        print()

        save = input(f"  Save to {result['name']}? [y/N]: ").strip().lower()
        if save == "y":
            _write_to_file(result["name"], result["source"])
            print(f"  Saved to {result['name']}\n")


def _print_generation_report(result: dict, validation=None):
    """Print a formatted generation report."""
    print(f"\n{'='*60}")
    print(f"  BSL Script Generation Report")
    print(f"{'='*60}")

    print(f"\n  Prompt:          {result['description']}")
    print(f"  Filename:        {result['name']}")
    print(f"  Dependencies:    {', '.join(result['dependencies']) or '(none)'}")
    print(f"  Arguments:       {', '.join(result['args']) or '(none)'}")

    if result.get("matched_example"):
        print(f"  Best match:      {result['matched_example']} "
              f"(score: {result['match_score']})")
    else:
        print(f"  Generation:      From scratch (no matching example)")

    if validation:
        print(f"  Validation:      {'PASSED' if validation.is_valid else 'HAS ISSUES'}")
        if not validation.is_valid:
            print(validation.detailed_report())


def _write_to_file(filepath: str, source: str):
    """Write the script to a file, ensuring .bsl extension."""
    if not filepath.endswith(".bsl"):
        filepath += ".bsl"
    try:
        with open(filepath, "w") as f:
            f.write(source)
        print(f"\n  Written to: {os.path.abspath(filepath)}")
    except IOError as e:
        print(f"\n  Error writing to {filepath}: {e}")


def _print_interactive_help():
    """Print help for interactive mode."""
    print()
    print("  Commands:")
    print("    <prompt>    Describe the script you want (e.g., 'create a docker cleanup script')")
    print("    quit        Exit interactive mode")
    print("    help        Show this help")
    print("    train       Show training corpus summary")
    print()


if __name__ == "__main__":
    main()
