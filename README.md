# LAFS Container Formatter

A high-performance, pure-Rust structural container formatter implementing the **Language-Agnostic Formatting Style (LAFS)** across systems languages, web applications, mobile platforms, scripting environments, configuration manifests, and technical documentation.

Official Universal AI Agent Skill: [lafs-formatting](https://github.com/Winstead-Inc/lafs-formatting)

---

> [!IMPORTANT]
> **This Repository is NOT a Cesspool of ugly, improperly formatted K&R Brain Rot & Slop.**
> Trailing delimiter and container attachments (`fn() {`, `if (Condition) {`), asymmetrical diagonal vector tracking, and compressed scope exit clusters (`}}});`) are strictly banned across all languages, files, templates, scripts, and documentation. Delimiters are load-bearing architectural columns, not sentence punctuation.

---

> [!WARNING]
> **Any commit or pull request that does not obey this rule will be reformatted automatically and committed back to the repository without the committer's explicit knowledge or consent.**
> If you push improperly formatted code, automated GitHub CI bots and repository hooks will rewrite the code into compliant LAFS geometry and push it directly onto your branch without your consent.

---

## Architectural Principles

LAFS treats code containers (`()`, `{}`, `[]`, `<>`) as structural rooms whose boundaries form load-bearing columns. Every delimiter pair is evaluated against a deterministic three-tier escalation hierarchy:

1. **State 1 — The Monolith (Horizontal Density)**:
   - When expressions, parameter lists, or atomic blocks fit within the Canvas width (default: 160 columns, configurable via `-c` / `--canvas-width`) and satisfy the Complexity Budget (at most 4 statements, nesting depth at most 2, zero internal line breaks or comments), they reside on a single horizontal line (`Foo(A, B)`, `int Add(int A, int B) { return A + B; }`).
2. **State 2 — Container Grouping / Simplified Symmetry (Compound Delimiters)**:
   - When nested containers form a **Sole-Child Chain** without intervening statements or expressions (such as `[{( ... )}]` or `({ ... })`), consecutive opening delimiters collapse horizontally into a compound boundary at the base indentation level, with mirrored closing runs on a dedicated line (`)}]`).
3. **State 3 — Expanded Symmetry / Hierarchical Architecture (Load-Bearing Columns)**:
   - When control flow, multi-line statements, or canvas breaches occur, delimiters decouple onto dedicated lines.
   - **Zero End-of-Line Hugging**: Opening delimiters never hang at the end of a statement line (`FunctionName(` or `if (X) {` are strictly prohibited).
   - **Translational Symmetry**: Opening and closing delimiters share the exact same column coordinate on the $X$-axis ($\text{Column}(\text{Bound}_{\text{Open}}) \equiv \text{Column}(\text{Bound}_{\text{Close}})$).
4. **Column-Anchored Hybrid (Section 6.1)**:
   - For language grammars where Automatic Semicolon Insertion (ASI) or compiler constraints penalize detached openers after control statements (`return (`, `throw (`, `yield (` in JavaScript/TypeScript, or Go control blocks), LAFS anchors the opening delimiter on the statement line while maintaining strict vertical closure symmetry on a dedicated line. Can be toggled via `--cah`.

---

## Supported Languages & File Types

LAFS provides native, AST/CST-level lexing, boundary pairing, and formatting across 16+ programming languages, configuration syntaxes, and markup formats:

| Language Family             | File Extensions                                                                             | Key Capabilities & Highlights                                                                                                                                                                                                                                                                                                                |
| :-------------------------- | :------------------------------------------------------------------------------------------ | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rust**              | `.rs`                                                                                     | Structs, enums, pattern matching, match guards, closures, raw strings (`r#"..."#`), byte literals, macros, attributes, HRTB lifetimes, turbofish `::<T>`, and Cargo manifests/lockfiles (`Cargo.toml`, `Cargo.lock`).                                                                                                                |
| **Typst**             | `.typ`                                                                                    | Full native AST/CST-level lexing, boundary pairing, and formatting for Typst markup, scripting, and vector typesetting. Supports `#` code transitions without stray space injection, nested block comments (`/* /* ... */ */`), hyphenated identifiers (`content-base`, `extra-chapters`, `starts-with`), labels (`<sec:intro>`), empty dictionaries `(:)` and arrays `()`, math equations (`$...$`, `math.equation`), parameter lists, and detached Allman braces/brackets/parentheses. |
| **TypeScript & TSX**  | `.ts`, `.tsx`, `.mts`, `.cts`                                                       | AST/CST disambiguation of `<const T,>` generic arrow parameters vs JSX opening tags, relational `<`/`>` operators vs JSX tags, multiline JSX elements with full HTML tree indentation, attribute `=` vertical column alignment (LAFS Section 6.3), alloyed style props `style={{ ... }}`, and Column-Anchored Hybrid `return (`. |
| **JavaScript & JSX**  | `.js`, `.jsx`, `.mjs`, `.cjs`                                                       | ESNext features, JSX tree indentation, render-prop callbacks, IIFE invocation closure `)()`, object destructuring, and template literals.                                                                                                                                                                                                  |
| **Dart & Flutter**    | `.dart`                                                                                   | Primary constructors, cascade notation `..`, switch expressions, records, single-element tuples `(x,)`, collection `if`/`for`, dot shorthands, and generic typedefs.                                                                                                                                                                 |
| **Kotlin**            | `.kt`, `.kts`                                                                           | Two dedicated engines: Version 1 (Pure Allman style for all containers) and Version 2 (Official Kotlin compiler compatible / Brain K&R). Data classes, `when` expressions, trailing lambdas, backtick identifiers.                                                                                                                         |
| **Markdown**          | `.md`, `.markdown`, `.mdown`, `.mkd`                                                | Extracts and formats embedded code snippets across all languages inside fenced code blocks (`` ``` `` or `~~~`), while preserving all non-code content (prose, non-code fences like mermaid/math, lists, tables, blockquotes) without artificial 80-column line breaks.                                                                    |
| **C & C++**           | `.c`, `.h`, `.cpp`, `.hpp`, `.cc`, `.cxx`, `.hh`, `.hxx`, `.inl`, `.cu` | Preprocessor directives (`#include`, `#define`, `#if`, `#endif`), template classes, nested generic closer splitting (`>>` vs `> >`), namespaces, pointers/references, and raw string literals `R"(...)"`.                                                                                                                      |
| **C#**                | `.cs`                                                                                     | Properties, lambda expressions, LINQ queries, pattern matching, records, verbatim strings `@"..."`, and raw strings `"""..."""`.                                                                                                                                                                                                         |
| **Go**                | `.go`                                                                                     | Goroutines, select blocks, multiple return signatures, raw string literals `` `...` ``, and Column-Anchored Hybrid control blocks.                                                                                                                                                                                                           |
| **Shell**             | `.sh`, `.bash`, `.zsh`, `.ksh`                                                      | Function declarations, conditional blocks `[[ ... ]]`, arithmetic expansions, subshells `(...)`, line continuations, and heredoc preservation.                                                                                                                                                                                           |
| **HTML & Polyglot**   | `.html`, `.htm`, `.vue`, `.svelte`                                                  | Full HTML tree indentation, embedded `<script>` (JS/TS) and `<style>` (CSS) formatting without corrupting surrounding markup.                                                                                                                                                                                                            |
| **PHP**               | `.php`, `.phtml`                                                                        | Standalone PHP code and mixed PHP embedded within HTML template structures.                                                                                                                                                                                                                                                                  |
| **CSS / SCSS / LESS** | `.css`, `.scss`, `.less`                                                              | Rulesets, media queries `@media`, CSS custom properties (variables), nested rules, and uppercase hex color token enforcement (`#7100FF`).                                                                                                                                                                                                |
| **JSON**              | `.json`, `.jsonc`, `.json5`                                                           | Pure Allman and Monolith formatting for nested objects and arrays with comments preservation.                                                                                                                                                                                                                                                |
| **TOML & Lockfiles**  | `.toml`, `.lock`                                                                        | Key-value pairs, table arrays `[[...]]`, inline tables, multiline arrays, Allman brackets, and alloyed groupings.                                                                                                                                                                                                                          |
| **Batch / CMD**       | `.bat`, `.cmd`                                                                          | Windows batch scripts, labels, loops, and commands.                                                                                                                                                                                                                                                                                          |

---

## Safety Gate & Verification

Every formatting operation passes through a strict verification pipeline before being written:

* **Token Stream Equivalence**: Verifies non-whitespace token stream equivalence between input and output to guarantee zero data loss, zero altered identifiers, and zero dropped expressions.
* **Mathematical Idempotence**: Guarantees that formatting an already-formatted file yields the exact same content:
  $$
  \text{Format}(\text{Format}(\text{Source})) \equiv \text{Format}(\text{Source})
  $$
* **Fail-Safe Mechanism**: If any parse ambiguity, unbalanced container, or lexer error occurs, the formatter safely fails open, reports diagnostic context to `failed.txt` (or custom log path), and leaves the original file completely untouched.

---

## Installation & Native Binaries

Precompiled standalone binaries with zero external runtime dependencies are automatically published on [GitHub Releases](https://github.com/Winstead-Inc/Language-Agnostic-Formatting-Style/releases):

| Platform                             | Binary Asset           | Description                                                                                                                          |
| :----------------------------------- | :--------------------- | :----------------------------------------------------------------------------------------------------------------------------------- |
| **Windows (x86_64)**           | `LAFS.exe`           | Native 64-bit Windows executable                                                                                                     |
| **Linux (Any Distro, x86_64)** | `LAFS-linux-x86_64`  | Fully static `musl` binary. Zero glibc dependency; runs on Alpine, Ubuntu, Debian, RHEL, CentOS, Fedora, Arch, NixOS, Amazon Linux |
| **Linux (ARM64 / aarch64)**    | `LAFS-linux-aarch64` | Fully static `musl` binary for 64-bit ARM (Raspberry Pi 4/5, AWS Graviton, Apple Silicon VMs)                                      |
| **macOS (Universal 2)**        | `LAFS-darwin`        | Universal binary combining Apple Silicon (`arm64`) and Intel (`x86_64`) native code                                              |
| **macOS (Apple Silicon)**      | `LAFS-darwin-arm64`  | Standalone Apple Silicon M1/M2/M3/M4 binary                                                                                          |
| **macOS (Intel)**              | `LAFS-darwin-x86_64` | Standalone Intel Mac binary                                                                                                          |

### Quick Start (Linux / macOS)

```bash
# Make executable
chmod +x LAFS-linux-x86_64 # or LAFS-darwin

# Move to your PATH
mv LAFS-linux-x86_64 /usr/local/bin/lafs # or ~/.local/bin/lafs

# Verify installation
lafs --version
```

### Build from Source

```bash
cargo build --release
```

The compiled binary will be located at `target/release/Language-Agnostic-Formatting-Style` (or `target/release/LAFS.exe` on Windows).

---

## CLI Usage & Options

### Complete CLI Options Reference

```text
lafs [OPTIONS] <FILES... | DIRECTORIES...>
```

| Flag                                                                | Argument      | Description                                                                                                                                   | Default                                        |
| :------------------------------------------------------------------ | :------------ | :-------------------------------------------------------------------------------------------------------------------------------------------- | :--------------------------------------------- |
| **`-w`**, **`--write`**                             | None          | Overwrite files in-place with formatted output                                                                                                | Enabled by default                             |
| **`-o`**, **`--output`**, **`--out`**       | `<PATH>`    | Output file or destination directory                                                                                                          | Appends `.formatted` if passed without value |
| **`--out-dir`**, **`--output-dir`**                 | `<DIR>`     | Destination directory (replicates source directory tree)                                                                                      | None                                           |
| **`--prefix`**                                              | `[PREFIX]`  | Prepend prefix to output filename stem                                                                                                        | `formatted_` (if flag passed without value)  |
| **`--suffix`**                                              | `[SUFFIX]`  | Append suffix to output filename stem before extension                                                                                        | `.formatted` (if flag passed without value)  |
| **`-f`**, **`--file`**, **`--folder`**      | `<PATH...>` | Format specified file(s) or folder(s) (comma-separated or repeated)                                                                           | None                                           |
| **`-d`**, **`--dir`**, **`--directory`**    | `<DIR...>`  | Explicit directory target                                                                                                                     | None                                           |
| **`-a`**, **`--all`**                               | None          | Format all eligible files in current directory and all subdirectories                                                                         | False                                          |
| **`-e`**, **`--ext`**                               | `<EXT...>`  | Only format files with specific extension(s) (e.g. `-e rs,ts,md`)                                                                           | All supported extensions                       |
| **`--all-types`**                                           | None          | Format all text files regardless of extension (excluding binary/docs)                                                                         | False                                          |
| **`-x`**, **`--exclude`**                           | `<PATTERN>` | Exclude files or folders matching wildcard patterns (e.g. `-x target -x vendor`)                                                            | Default exclusions                             |
| **`--cah`**                                                 | `<-1\|0\|1>`  | Enable (`1`) or disable (`-1`/`0`) Column-Anchored Hybrid (Section 6.1)                                                                 | `1` (Enabled)                                |
| **`--unwrap-paragraphs`**, **`--no-abrupt-breaks`** | None          | Reflow Markdown paragraphs into continuous semantic blocks (strips archaic 80-col breaks)                                                     | False                                          |
| **`--no-unwrap-paragraphs`**                                | None          | Preserve original line breaks in Markdown paragraphs                                                                                          | Active                                         |
| **`--kotlin-version`**                                      | `<1\|2>`     | Kotlin formatting profile: Version 1 (Pure Allman) or Version 2 (Official)                                                                    | `1` (Version 1)                              |
| **`--kotlin-style`**                                        | `<STYLE>`   | Kotlin style alias: `allman` (Version 1) or `official` (Version 2)                                                                        | `allman`                                     |
| **`--allman-kotlin`**                                       | None          | Enforce Version 1 (Pure Allman style for all containers)                                                                                      | Active                                         |
| **`--official-kotlin`**                                     | None          | Enforce Version 2 (Official Kotlin compiler compatible / Brain K&R)                                                                           | Inactive                                       |
| **`-c`**, **`--canvas-width`**                      | `<WIDTH>`   | Maximum column width before vertical expansion                                                                                                | `160`                                        |
| **`-i`**, **`--indent`**                            | `<SIZE>`    | Indentation size in spaces                                                                                                                    | `4`                                          |
| **`-t`**, **`--tabs`**                              | None          | Use hard tabs instead of spaces for indentation                                                                                               | Spaces                                         |
| **`-l`**, **`--language`**                          | `<LANG>`    | Force language mode (`c`, `cpp`, `cs`, `js`, `ts`, `jsx`, `tsx`, `sh`, `kt`, `dart`, `rust`, `md`, `html`, `css`, `typ`) | Auto-detect                                    |
| **`--preserve-empty-lines`**                                | `<0\|1>`     | Preserve existing intentional empty lines (`1`, default on ALL supported and generic files) or strip them (`0`)           | `1` (Preserve)                               |
| **`--no-preserve-empty-lines`**                             | None          | Strip redundant empty lines across all formatted files (alias for `--preserve-empty-lines 0`)                                | False                                          |
| **`-mbl`**, **`--max-blank-lines`**                 | `<NUM>`     | Maximum consecutive blank lines to preserve                                                                                                   | `1`                                          |
| **`--max-statements`**                                      | `<NUM>`     | Maximum statements permitted in a single-line monolith block                                                                                  | `4`                                          |
| **`--check`**                                               | None          | Dry-run check: exits with `0` if clean, `1` if unformatted                                                                                | False                                          |
| **`format failed`**                                         | None          | Re-run formatting only on files recorded in `failed.txt`                                                                                    | False                                          |
| **`-of`**, **`--output-failed`**                    | `<FILE>`    | Custom path for logging unformatted or failed file paths                                                                                      | `failed.txt`                                 |
| **`--no-log`**                                              | None          | Disable automatic timestamped session and error log files                                                                                     | False                                          |
| **`-j`**, **`--threads`**                           | `<COUNT>`   | Worker thread count for parallel multi-file processing                                                                                        | Available CPU cores                            |
| **`-y`**, **`--yes`**                               | None          | Skip interactive confirmation prompts                                                                                                         | False                                          |
| **`--verbose`**                                             | None          | Display verbose status for clean (unchanged) files                                                                                            | False                                          |
| **`--stdout`**                                              | None          | Emit formatted content directly to standard output                                                                                            | False                                          |
| **`--stdin-filepath`**                                      | `<PATH>`    | Virtual filepath for language detection when formatting stdin                                                                                 | None                                           |
| **`-h`**, **`--help`**                              | None          | Display usage information                                                                                                                     | None                                           |
| **`-v`**, **`--version`**                           | None          | Display version information                                                                                                                   | None                                           |

---

### Usage Workflows & Examples

#### 1. In-Place Formatting (Default)

```bash
# Format specific files in-place
lafs -w src/main.rs src/lib.rs

# Format an entire directory recursively
lafs src -y

# Format all files in current project
lafs -a -y
```

#### 2. Outputting to Another File or Destination

```bash
# Output single formatted file to a specific destination file
lafs input.rs -o output.rs

# Replicate an entire directory tree into a destination directory
lafs src -o dist -y
# Structure replicated: src/a/b.rs -> dist/a/b.rs
```

#### 3. Non-Destructive Formatting with Prefix or Suffix

```bash
# Save formatted copy alongside original with suffix (original untouched)
lafs src/main.rs --suffix
# Outputs: src/main.formatted.rs

# Save formatted copy with prefix
lafs src/main.rs --prefix
# Outputs: src/formatted_main.rs

# Custom prefix and suffix combined with output directory
lafs src -o dist --prefix f_ --suffix .lafs -y
# Outputs: dist/models/f_user.lafs.rs
```

#### 4. Formatting Embedded Code in Markdown Files

```bash
# Format all code blocks inside a Markdown document
lafs docs/README.md -o docs/README.clean.md

# Format all Markdown files in a directory
lafs docs/ -e md -y

# Format Markdown and unwrap abrupt mid-paragraph line breaks into continuous prose
lafs docs/README.md --unwrap-paragraphs
```

#### 5. Formatting Typst Documents (`.typ`)

```bash
# Format a Typst document in-place
lafs document.typ

# Format an entire directory of Typst files
lafs -e typ -d docs/

# Verify that all Typst documentation files are clean in CI
lafs --check -e typ -d docs/
```

#### 6. Automated Changelog Generator (`lafs changelog`)

```bash
# Generate or update CHANGELOG.md from git commit history
lafs changelog

# Generate changelog with custom release version and custom destination
lafs changelog -v 1.2.0 -o RELEASE_NOTES.md

# Preview generated changelog to stdout without modifying files
lafs changelog --stdout
```

#### 7. Controlling Column-Anchored Hybrid (CAH)

```bash
# Enforce pure Allman geometry everywhere (even on return / closures)
lafs --cah -1 src/app.tsx

# Enable Column-Anchored Hybrid for syntax-sensitive grammar edge cases (default)
lafs --cah 1 src/app.tsx
```

#### 8. Kotlin Formatting Engine Selection

```bash
# Enforce Version 1 (Pure Allman style for all Kotlin containers)
lafs --allman-kotlin src/App.kt

# Enforce Version 2 (Official Kotlin compiler compatible / Brain K&R)
lafs --official-kotlin src/App.kt
```

#### 9. Shallow vs. Recursive Directory Targeting

```bash
# Shallow / Non-recursive (trailing slash): formats ONLY files directly in scratch
lafs -f "scratch/"

# Recursive (no trailing slash): formats scratch and all its subdirectories
lafs -f "scratch"
```

#### 10. Stdin / Stdout Streaming (Editor Integration)

```bash
# Stream code via pipe
cat MyFile.cpp | lafs -

# Format stdin with virtual path for language detection
cat MySnippet.tsx | lafs --stdin-filepath app.tsx -
```

#### 11. CI/CD Automated Verification Gate

```bash
# Check if any repository file requires formatting (exits 1 on unformatted)
lafs --check --no-log -a
```

#### 12. Retrying Failed Files

```bash
# Re-runs formatting on all files logged in failed.txt
lafs format failed
```

---

## LAFS-Compliant Cargo Toolchain (`stable-x86_64-pc-windows-msvc`)

### Background & Architectural Problem

Stock `cargo.exe` enforces standard TOML 1.0 ABNF grammar (`keyval-sep = ws %x3D ws`), where a newline immediately following an assignment operator (`=`) is treated as a hard lexer syntax error:

```text
error: key with no value, expected `=`
 --> Cargo.toml:10:8
```

This constraint prevents developers from formatting TOML manifests and lockfiles according to pure Allman / LAFS symmetry (`dependencies =\n[\n ... \n]`).

### The Native Compiler Solution

A custom, LAFS-compliant release binary of `cargo.exe` has been compiled and integrated directly into your global Rustup toolchain (`stable-x86_64-pc-windows-msvc`). It eliminates the K&R delimiter restriction without fragile shims or on-the-fly file rewriting.

#### Key Features:

- **Pure LAFS Manifest Support:** Natively parses `Cargo.toml` and `.cargo/config.toml` containing monolithized or symmetric delimiter blocks:
  ```toml
  [dependencies]
  proc-macro2 = ["1.0",]

  quote = { version = "1.0", }
  ```
- **Pure LAFS Lockfile Generation:** Cargo's lockfile serializer (`emit_package` in `ops/lockfile.rs`) automatically writes `Cargo.lock` using pure vertical symmetry for multiline dependencies:
  ```toml
  [[package]]
  name = "quote"
  version = "1.0.47"
  dependencies = ["proc-macro2",]
  ```
- **Global Operating System Integration:**
  - Works natively with **Antigravity IDE** and **VS Code**.
  - Works seamlessly with **`rust-analyzer`** in background language server diagnostics.
  - Works across all command shells (**PowerShell**, **CMD**, **Bash**, **Git Bash**).
  - Survives subprocess calls, build scripts, and Git hooks.
