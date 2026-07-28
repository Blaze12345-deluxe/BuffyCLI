================================================================================
                     BSL AI Script Generator
================================================================================

Location: AI/
Language: Python 3
Purpose:  Train an AI to generate valid BSL scripts from natural language
          prompts and validate them for correct syntax.


================================================================================
QUICK START
================================================================================

  Generate a script from a prompt:
    python AI/run.py "create a python virtual environment"

  Generate and save to a file:
    python AI/run.py "backup my home directory" --output backup.bsl

  Generate with syntax validation:
    python AI/run.py "check disk space" --check

  Interactive mode:
    python AI/run.py --interactive

  Show training corpus summary:
    python AI/run.py --train

================================================================================
FILE DESCRIPTIONS
================================================================================

  run.py              Main CLI entry point. Parses arguments, orchestrates
                      generation and validation.

  bsl_generator.py    Core generation engine. Takes a natural language prompt,
                      matches against training examples, adapts or generates
                      from scratch. Returns complete, valid .bsl source.

  bsl_tester.py       Syntax validation engine. Rule-based validation checks
                      metadata placement, instruction syntax, argument quoting,
                      variable usage, dangerous commands, and script structure.
                      Optionally integrates with 'buffy --check' binary.

  bsl_train.py        Training pipeline. Loads the corpus, analyzes patterns,
                      exports training data as JSON for external AI systems,
                      and generates format reference documents.

  training_data.py    Training corpus. Contains BSL example scripts organized
                      by category with descriptions, keywords, tags, and
                      dependencies. Also defines pattern templates for
                      generation.


================================================================================
HOW IT WORKS
================================================================================

1. PROMPT ANALYSIS
   The user provides a natural language prompt (e.g., "create a script
   that monitors disk space").

2. PATTERN MATCHING
   The generator scores each training example against the prompt using
   keyword matching, tag matching, and description overlap. The best
   match is selected.

3. SCRIPT GENERATION
   - If a good match is found (score >= 10): the matched example's
     source is adapted (metadata updated, arguments adjusted).
   - If no good match exists: a script is generated from scratch using
     the pattern templates and detected shell commands.

4. SYNTAX VALIDATION
   The generated script is validated against BSL spec rules:
   - Metadata placement (must come before instructions)
   - Instruction validation (WRITE, RUN, WAIT, CLEAR, EXIT)
   - OUTPUT toggle validation
   - Argument quoting
   - Variable usage
   - Dangerous command detection
   - Optional integration with 'buffy --check'


================================================================================
TRAINING CORPUS
================================================================================

The training corpus contains 12 example scripts covering these categories:

  system          - system-info, system-update, disk-usage
  development     - pip-env, project-setup, git-quick-setup
  network         - network-diagnostic
  containers      - docker-cleanup
  backup/archive  - backup-directory
  files/download  - download-file, find-large-files

Each example includes:
  - Full BSL source code
  - Natural language description
  - Keyword list for prompt matching
  - Tags for categorization
  - Shell dependencies
  - Expected arguments

Export the corpus for external AI systems:
  python AI/bsl_train.py --export bsl-corpus.json


================================================================================
COMMAND REFERENCE
================================================================================

  python AI/run.py <prompt>
    Generate a BSL script from a natural language prompt.

  python AI/run.py <prompt> --output <file>
    Generate and save to a specific file.

  python AI/run.py <prompt> --check
    Generate and validate syntax.

  python AI/run.py <prompt> --buffy-check
    Generate and validate using both rules and 'buffy --check'.

  python AI/run.py --interactive
    Interactive prompt loop for multiple generations.

  python AI/run.py --train
    Show training corpus summary.

  python AI/bsl_tester.py <file.bsl>
    Validate a .bsl file for syntax errors.

  python AI/bsl_train.py --export [file.json]
    Export training corpus as JSON.

  python AI/bsl_train.py --export-format
    Print BSL format reference.


================================================================================
REQUIREMENTS
================================================================================

  - Python 3.6+ (no external dependencies required)
  - Optional: 'buffy' binary in PATH for full validation

================================================================================
