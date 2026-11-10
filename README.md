# LAFS Container Formatter

A pure-Rust, high-performance container formatter implementing the **Language-Agnostic Formatting Style (LAFS)** for **C**, **C++**, **C#**, **Go**, **JavaScript**, **TypeScript**, **PHP**, **HTML**, and **CSS**.

---
> [!IMPORTANT]
> **This Repository is NOT a Cesspool of ugly, improperly formatted K&R Brain Rot & Slop.**
> Trailing delimiter/container attachments (`fn() {`, `if (Condition) {`), asymmetrical diagonal vector tracking, and compressed scope exit clusters (`}}});`) are strictly banned across all languages, files, templates, scripts, and documentation. Delimiters are load-bearing architectural columns, not sentence punctuation.
---
> [!WARNING]
> **Any commit or pull request that does not obey this rule will be reformatted automatically and committed back to the repository without the committer's explicit knowledge or consent.**
> If you push improperly formatted code, automated GitHub CI bots and repository hooks will rewrite the code into compliant LAFS geometry and push it directly onto your branch without your consent.

---

## Architectural Principles

LAFS treats code containers (`()`, `{}`, `[]`, `<>`) as structural rooms whose boundaries form load-bearing columns. Every container is evaluated against a deterministic three-tier escalation hierarchy:

1. **State 1 - The Monolith (Horizontal Density)**:
   - When expressions, simple signatures, or atomic blocks fit within the Canvas width (default: 120 columns) and contain zero internal line breaks or comments, they are preserved as single-line horizontal units (`Foo(A, B)`, `int Add(int A, int B) { return A + B; }`).
2. **State 2 - Brace Grouping / Simplified Symmetry (Compound Delimiters)**:
   - When containers form a **Sole-Child Chain** without intervening logic (such as `[{( ... )}]` or `({ ... })`), their delimiters fuse into compound boundaries on a single line at the base indent, with mirrored closing runs (`)}];`).
3. **State 3 - Expanded Symmetry / Hierarchical Architecture (Load-Bearing Columns)**:
   - When control flow, multi-line statements, or canvas breaches occur, delimiters decouple onto dedicated lines.
   - **Zero End-of-Line Hugging**: Opening delimiters never hang at the end of a line (`FunctionName(` or `if (X) {` are strictly prohibited).
   - **Translational Symmetry**: Opening and closing delimiters share the exact same column coordinate.

---

## Features

- **Full Multi-Language Support**:
  - Handles **C / C++ / C# / Go / JS / TS / PHP / CSS**.
  - **Polyglot Files**: Intelligently parses mixed-content files (e.g., HTML containing `<script>`, `<style>`, and `<?php ... ?>` blocks) and formats each language chunk individually without breaking the surrounding HTML.
  - Preprocessor directives (`#include`, `#define`, `#if`, `#endif`, `#region`) preserved intact.
  - String literals (standard `"..."`, C++ raw strings `R"(...)"`, C# verbatim `@"..."`, C# raw `"""..."""`, Go raw strings `` `...` ``, and character literals).
  - Single-line comments (`//`) and multiline block comments (`/* ... */`) preserved without reformatting corruption.
  - Template and generic angle brackets (`<T, U>`) disambiguated using follow-token and context rules.
- **Safety Gate**:
  - Automatically verifies non-whitespace token stream equivalence between input and output.
  - Enforces idempotence (`Format(Format(Source)) == Format(Source)`).
  - Fails open rather than corrupting source code.
- **Zero External C Dependencies**:
  - Implemented in 100% pure Rust.
  - Builds instantly on Windows, Linux, and macOS without requiring MSVC or GCC build tools.

---

## Build & Installation

```powershell
cd "b:\Language-Agnostic Formatting Style"
cargo build --release
```

The compiled binary will be located at:
`b:\Language-Agnostic Formatting Style\target\release\LAFS.exe`

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

- **Pure Allman Manifest Support:** Natively parses `Cargo.toml` and `.cargo/config.toml` containing detached delimiter blocks:
  ```toml
  [dependencies]
  proc-macro2 =
  [
      "1.0",
  ]

  quote =
  {
      version = "1.0",
  }
  ```
- **Pure Allman Lockfile Generation:** Cargo's lockfile serializer (`emit_package` in `ops/lockfile.rs`) automatically writes `Cargo.lock` using pure vertical symmetry:
  ```toml
  [[package]]
  name = "quote"
  version = "1.0.47"
  dependencies =
  [
      "proc-macro2",
  ]
  ```
- **Global Operating System Integration:** Because the patch is installed at the binary level in the active toolchain:
  - Works natively with **Antigravity IDE** and **VS Code**.
  - Works seamlessly with **`rust-analyzer`** in background language server diagnostics.
  - Works across all command shells (**PowerShell**, **CMD**, **Bash**, **Git Bash**).
  - Survives subprocess calls, build scripts, and Git hooks.

### Toolchain File Paths

| Component                         | Path                                                                                              |
| :-------------------------------- | :------------------------------------------------------------------------------------------------ |
| **Active Toolchain Binary** | `C:\Users\0.USS-OPPENHEIMER\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe`     |
| **Stock Cargo Backup**      | `C:\Users\0.USS-OPPENHEIMER\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe.bak` |
| **Cargo Source Repository** | `b:\cargo\`                                                                                     |
| **Compiled Release Binary** | `b:\cargo\target\release\cargo.exe`                                                             |

### Rebuilding & Updating the Toolchain

If you update or pull new Cargo features in `b:\cargo\`, recompile and re-deploy using:

```powershell
cd b:\cargo
cargo build --release --bin cargo
Copy-Item "target\release\cargo.exe" "$HOME\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe" -Force
```

---

## CLI Usage

### Formatting Files In-Place

```powershell
lafs -w file.c file.cpp file.cs file.php index.html
```

### Formatting Specified Files or Folders (`-f`, `--file`, `--folder`, `-d`, `--dir`)

Format specified file(s) or folder(s) directly:

```powershell
# Single file
lafs -f file.c

# Multiple files with comma separation
lafs -f file.c,file.cpp

# Multiple flags
lafs -f file.c -f file.cs

# User specified folder (shallow / non-recursive with trailing slash)
lafs -f "E:\Typst\scratch\"

# User specified folder (recursive across subdirectories without trailing slash)
lafs -f "E:\Typst\scratch"

# Positional folder path with specific language or extension filter
lafs -e js "E:\Typst\scratch\"
lafs -l js "E:\Typst\scratch"

# Explicit directory flag (-d / --dir / --directory)
lafs -d "E:\Typst\scratch\" -e js
```

> **Trailing Slash Differentiation for Folders**:
>
> - Paths ending with a trailing slash (`/` or `\`, e.g. `"scratch\"` or `"scratch/"`) target only the files located **directly inside that folder** (shallow / non-recursive, without descending into subdirectories).
> - Paths without a trailing slash (e.g. `"scratch"`) target the folder and **all its subdirectories recursively**.

### Checking Files and Logging Failures

Exits with code `0` if all files are clean, or `1` if any file requires formatting. Output failed and unformatted paths to a specific log file:

```powershell
lafs --check --output-failed failed.txt .
```

### Stdin / Stdout Streaming (Editor / Pipe Mode)

```powershell
type MyFile.cpp | lafs -
```

### Custom Canvas Width & Indentation

```powershell
lafs --canvas-width 160 --indent 4 -w MyFile.cs
```

### Retrying Failed Files (`lafs format failed`)

Re-runs formatting on all files recorded in `failed.txt` (or custom `-of <FILE>`):

```powershell
lafs format failed
```

If all previously failed files succeed, `failed.txt` is automatically cleared.

### Automated Terminal & Error Logs

During execution, LAFS automatically logs all console output and errors with generation timestamps:

- `lafs {YYYY-MM-DD HH-MM-SS}.log`: All console output produced during the session.
- `lafs error {YYYY-MM-DD HH-MM-SS}.log`: Captured error diagnostics (generated whenever errors occur).

To disable automatic file logging, supply `--no-log`:

```powershell
lafs --no-log -w .
```

### CLI Reference

| Flag                                | Description                                                                      |
| ----------------------------------- | -------------------------------------------------------------------------------- |
| `-w`, `--write`                 | Overwrite files in-place with formatted output                                   |
| `-f`, `--file <FILES>`          | Format only the specified file(s) (comma-separated or multiple)                  |
| `--check`                         | Check if files are formatted (exit code 1 if unformatted)                        |
| `format failed`                   | Reformat all files listed in `failed.txt`                                      |
| `-of`, `--output-failed <FILE>` | Log paths (with language and reason) of files that failed or require formatting  |
| `--no-log`                        | Disable automatic log file generation                                            |
| `-c`, `--canvas-width <WIDTH>`  | Maximum column width before vertical expansion (default: 160)                    |
| `-i`, `--indent <SIZE>`         | Indentation size in spaces (default: 4)                                          |
| `-t`, `--tabs`                  | Use tabs instead of spaces for indentation                                       |
| `-l`, `--language <LANG>`       | Force language mode (`c`, `cpp`, `cs`, `go`, `php`, `html`, `css`) |
| `--stdin-filepath <PATH>`         | Virtual file path when formatting from stdin                                     |
| `-h`, `--help`                  | Show usage help                                                                  |
| `-v`, `--version`               | Show version info                                                                |
