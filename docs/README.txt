================================================================================
                          BUFFY CLI - DOCUMENTATION INDEX
================================================================================

Project Name:  Buffy CLI Automation Framework
Description:   A lightweight, cross-platform CLI automation framework that
               lets anyone create and share terminal commands by writing
               simple text files (.bsl) -- no programming experience required.
Language:      Rust (single crate, binary + library)
Version:       0.1.0
License:       MIT
Repository:    https://github.com/Blaze12345-deluxe/BuffyCLI
Plugin Repo:   https://github.com/Blaze12345-deluxe/Buffy-Plugins

================================================================================
DOCUMENTATION CONTENTS
================================================================================

The docs/ folder contains the following plain-text files:

  README.txt             THIS FILE - Documentation index and reading guide.
  PROJECT_OVERVIEW.txt   Overall project vision, goals, and design principles.
  ARCHITECTURE.txt       Module architecture, dependency graph, data flow.
  CODEBASE.txt           Complete reference to every source file, struct,
                         function, and test.
  COMMANDS.txt           All CLI flags, built-in commands, and their usage.
  SCRIPT_LANGUAGE.txt    How .bsl files work - syntax, metadata, instructions,
                         variables, complete guide to writing scripts.
  MODULES.txt            Detailed documentation for each Rust module.
  DEVELOPMENT.txt        Development setup, build commands, coding standards.
  INSTALLATION.txt       How to install Buffy from source, cargo, or binary.
  CONFIGURATION.txt      All configuration files, environment variables, paths.
  ROADMAP.txt            Future plans, feature roadmap, version history.
  DESIGN_DECISIONS.txt   Key architectural and design decisions with rationale.
  TROUBLESHOOTING.txt    Common problems, diagnostics, repair procedures.
  CONTRIBUTING.txt       How to contribute code, docs, tests, or plugins.
  CHANGELOG.txt          Version history and release notes.
  GLOSSARY.txt           Definitions of terms, acronyms, and concepts.
  TODO.txt               Current tasks and planned work.
  KNOWN_ISSUES.txt       Known bugs, limitations, and workarounds.
  AI_CONTEXT.txt         All accumulated development context, design reasoning,
                         implementation notes, and assumptions.

================================================================================
RECOMMENDED READING ORDER
================================================================================

For NEW USERS (just want to use Buffy):

  1. INSTALLATION.txt       -- Get Buffy running on your system.
  2. COMMANDS.txt           -- Learn the CLI flags and how to run commands.
  3. SCRIPT_LANGUAGE.txt -- Understand .bsl files (the core concept).
  4. CONFIGURATION.txt      -- Learn about ~/.buffy/ and configuration.

For SCRIPT WRITERS (creating .bsl files):

  1. SCRIPT_LANGUAGE.txt -- Complete guide to writing scripts.
  2. PROJECT_OVERVIEW.txt   -- Understand the ecosystem.
  3. GLOSSARY.txt           -- Learn the terminology.

For PLUGIN DEVELOPERS (distributing packages):

  1. SCRIPT_LANGUAGE.txt       -- Master .bsl syntax first.
  2. MODULES.txt               -- Understand the package system.
  3. CONFIGURATION.txt         -- Learn about installed.json and repositories.
  4. TROUBLESHOOTING.txt       -- Common issues when publishing.

For RUST CONTRIBUTORS (modifying the Buffy codebase):

  1. PROJECT_OVERVIEW.txt     -- Understand the vision.
  2. ARCHITECTURE.txt         -- Module layout and dependencies.
  3. CODEBASE.txt             -- Deep dive into every module.
  4. MODULES.txt              -- Detailed module documentation.
  5. DESIGN_DECISIONS.txt     -- Why things are the way they are.
  6. DEVELOPMENT.txt          -- Build, test, and development workflow.
  7. CONTRIBUTING.txt         -- Pull request process and guidelines.
  8. AI_CONTEXT.txt           -- Complete development history and context.

For EVERYONE:
  READ ALL FILES. Each file is focused on a single topic and references
  others where relevant. The complete documentation set is designed so that
  a new developer -- or another AI agent -- can understand, maintain, and
  continue developing the project using only the docs/ folder and source code.

================================================================================
QUICK START
================================================================================

  1. Install Buffy:
       git clone https://github.com/Blaze12345-deluxe/BuffyCLI.git
       cd BuffyCLI
       cargo build --release
       sudo cp target/release/buffy /usr/local/bin/

  2. Install a package:
       buffy --install pip-env

  3. Run a command:
       buffy pip-env

  4. See all flags:
       buffy --help

================================================================================
FILE INDEX
================================================================================

Each file in docs/ covers exactly one topic. Cross-references to other files
are indicated by FILENAME.txt in UPPERCASE. All files are plain ASCII/UTF-8
text with no Markdown formatting.

  docs/README.txt             (this file)
  docs/PROJECT_OVERVIEW.txt
  docs/ARCHITECTURE.txt
  docs/CODEBASE.txt
  docs/COMMANDS.txt
  docs/SCRIPT_LANGUAGE.txt
  docs/MODULES.txt
  docs/DEVELOPMENT.txt
  docs/INSTALLATION.txt
  docs/CONFIGURATION.txt
  docs/ROADMAP.txt
  docs/DESIGN_DECISIONS.txt
  docs/TROUBLESHOOTING.txt
  docs/CONTRIBUTING.txt
  docs/CHANGELOG.txt
  docs/GLOSSARY.txt
  docs/TODO.txt
  docs/KNOWN_ISSUES.txt
  docs/AI_CONTEXT.txt

--- End of README.txt ---
