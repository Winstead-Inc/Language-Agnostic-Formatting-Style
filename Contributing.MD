# Contribution Guidelines

## 1. Absolute Structural Mandate: Strict LAFS Formatting Only

All code submitted to this repository **MUST STRICTLY be formatted in Language-Agnostic Formatting Style (LAFS)**.

> [!IMPORTANT]
> **This repository is NOT a cesspool of ugly, improperly formatted K&R slop.**
> Trailing delimiter/container attachments (`fn() {`, `if (Condition) {`), asymmetrical diagonal vector tracking, and compressed scope exit clusters (`}}});`) are strictly banned across all languages, files, templates, scripts, and documentation. Delimiters are load-bearing architectural columns, not sentence punctuation.

### Core Geometric Rules

1. **Zero End-of-Line Delimiter Hugging**: An opening delimiter or container must **never** cling to the tail of an execution statement or signature on multiline constructs.
2. **Translational Column Symmetry**: Opening and closing delimiters must occupy their own dedicated lines and align at the exact same horizontal coordinate ($X$-coordinate).
3. **The Monolith First Rule**: Flat expressions, atomic signatures, and compact blocks remain single-line horizontal monoliths only when they comfortably fit the canvas without internal branching or soft-wrapping (`int Add(int A, int B) { return A + B; }`).
4. **Container Grouping Architecture**: Chains of consecutive containers with a single child collapse into compound delimiters/containers on dedicated boundary lines to prevent vertical dispersion while preserving vertical pillar alignment.
5. **Universal Detached Backticks**: Multiline template literals and backtick containers must place opening and closing backticks on their own dedicated lines.
6. **Casing Hierarchy**:

   - `Pascal_Snake_Case`: Structs, Enums, Traits, Type Aliases.
   - `SCREAMING_SNAKE_CASE`: Constants and static allocations.
   - `PascalCase`: Functions, Methods, Variables, Modules, and Markup Attributes.

---

## 2. Policy on Non-Compliant Commits & Automated Normalization

> [!WARNING]
> **Any commit or pull request that does not obey this rule will be reformatted automatically and committed back to the repository without the committer's explicit knowledge or consent.**
> If you push improperly formatted code, automated GitHub CI bots and repository hooks will rewrite the code into compliant LAFS geometry and push it directly onto your branch.

To avoid automated bot rewrites of your branch, format your work locally before pushing:

```powershell
# Format all files in repository in-place
lafs -a -y

# Format specific file or folder
lafs -f "src\"
lafs -f path/to/file.rs

# Check formatting compliance without writing (returns exit code 1 if dirty)
lafs --check -a
```

---

## 3. GitHub Automation & Enforcement Setup

To implement automated LAFS formatting in GitHub, two mechanisms are available:

1. **GitHub Actions Auto-Format Workflow** (automatically formats non-compliant pushes and commits changes back to the branch).
2. **Local Git Pre-Commit Hook** (formats code on the developer machine before the commit is created).

### Option A: GitHub Actions Auto-Formatting Bot (`.github/workflows/lafs-auto-format.yml`)

Create `.github/workflows/lafs-auto-format.yml` in your repository:

```yaml
name: LAFS Auto-Format Automation

on:
  push:
    branches: [ "**" ]
  pull_request:
    branches: [ "**" ]

permissions:
  contents: write

jobs:
  auto-format:
    name: Enforce LAFS Formatting
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4
        with:
          ref: ${{ github.head_ref || github.ref }}
          fetch-depth: 0

      - name: Install Rust Toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Build LAFS Formatter Engine
        run: cargo build --release

      - name: Format Repository In-Place
        run: ./target/release/lafs -a -y -w

      - name: Commit and Push Reformatted Changes
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "chore(style): enforce strict LAFS formatting [skip ci]"
          commit_user_name: "github-actions[bot]"
          commit_user_email: "41898282+github-actions[bot]@users.noreply.github.com"
          commit_author: "LAFS Bot <actions@github.com>"
```

### Option B: Local Pre-Commit Hook (`.githooks/pre-commit`)

Ensure zero non-compliant code leaves the contributor's machine by configuring a pre-commit hook:

```bash
#!/bin/sh
# .githooks/pre-commit
echo "[LAFS] Formatting staged files before commit..."

# Format repository files
lafs -a -y -w

# Re-stage any files modified by LAFS
git update-index --again
```

Activate the hook in the repository:

```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
```
