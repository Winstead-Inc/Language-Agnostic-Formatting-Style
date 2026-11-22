# Changelog

All notable changes to the Language-Agnostic Formatting Style (LAFS) project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-20

### Added

- Implement HTML tree indentation for multiline JSX and TSX elements

- Add full JSX and TSX language formatting support with text mode and prop emission

- Preserve intentional empty lines by default across all language engines

- Add source formatter and language-specific formatters for batch, dart, go, js/ts, shell, and kotlin

- Implement unified profile-driven Container_Matcher per Container Matching Law §6

- Implement declarative Language_Profile per Container Matching Law §5.1 & §11

- Add K&R CESSPOOL BRAINROT SLOP CURE and SLOP formatting test files

- Format flat control flow headers as single-line monoliths

- Format generic typedefs with Allman angle brackets and monolith Function types

- Align IIFE closures, container delimiters and collection-control flow in LAFS symmetry

- Add --cah CLI options and fix cascade and control-flow collection indentation in Dart

- Add JS/TS formatter and language-agnostic formatting engine core modules

- Add HTML and CSS formatting with automatic polyglot detection and strict column-anchored hybrid

- Support template literals with \ AST containers, JSX, ASI column-anchored hybrid, and ES2026 private identifiers

- Add native battle-tested LAFS formatter for Dart/Flutter

- Compile and stage all native platform binaries in repository root

- Reformat Cargo.lock to pure column-anchored hybrid

- Enforce column-anchored hybrid over monolith for Cargo.lock dependencies

- Implement Language-Agnostic Formatting Style engine and automation pipeline

- Add code emitter and format tests for LAFS engine

- Implement language-agnostic formatting engine and CLI parser

- Implement core formatting engine and language-agnostic style layout planner

- Implement Kotlin support in Rust and Specification MD

- Multi-language code snippet extraction and formatting within Markdown documents while preserving prose and non-code blocks.

- Automated paragraph unwrapping for Markdown (`--unwrap-paragraphs`, `--no-abrupt-breaks`) to eliminate legacy 80-column line breaks.

- Comprehensive output options: destination file/directory routing (`-o`, `--out-dir`), prefix (`--prefix`), suffix (`--suffix`), and full directory tree replication.

- Built-in automated Changelog generator (`lafs changelog`).

### Fixed

- Resolve TSX parser and V8 hazards, arrow placement, and generic type formatting

- Isolate arrow function statement boundaries and preserve JSX text node spacing

- Switch generic type containers (<{, <) to Monolith only and enforce Monolith on const arrow functions without newlines before =>

- Enforce Monolith and Column-Anchored Hybrid on generic type definitions to prevent detached delimiter syntax errors

- Harden generic angle bracket resolution against false matches across statements and unmatched delimiters

- Support TypeScript generic type matrix, resolve JSX tag detection, and harden template literal spacing

- Resolve JSX closing tags, hyphenated attributes, and private identifier spacing

- Resolve multiline string interpolation, record generics, and ternary spacing

- Multiline string interpolation, maximal container grouping, and syntax symmetry

- Use PeekAhead character lookahead for C# and C++ raw strings to prevent non-ASCII UTF-8 boundary panics

- Resolve token verification edge cases for unclosed delimiters, shebangs, and multiline literals

- Resolve Rust failure analysis cases in lexer, emitter, and container tree

### Changed

- Antidote Reference for Battle Testing Dart

- Organize language formatters into dedicated directories under src/languages/

- Remove empty unused file

- Stage and commit root binary LAFS.exe alongside release

- Add cross-platform native release matrix for macOS universal and Linux static musl

- Remove toolchain file paths and release only LAFS.exe binary

- ...

- Publish immutable-compliant release using run-number versioning and --latest flag

- Use gh release cli for atomic immutable-safe release publishing

- Add automated release workflow for Language-Agnostic Formatting Style.exe

- Restructure project layout with modular rust source files and tests

- Style: fix indentation in JavaScript template literal example

- New

- Screw K&R SLop

### Documentation

- Add Container Matching Law specification for LAFS container engine

- Add zip external_attr permission mandate for VSIX packaging

- Style: align options table in README

- Add extension creation and VSIX packaging enforcement protocol

- Hardcode Column-Anchored Hybrid rule for lockfiles in specification

- Add link to official lafs-formatting skill repo above important disclaimer

- Add horizontal rules around important warning notice in README

- Add warning about automated code reformatting for non-compliant commits

- Update anti-K&R slop policy in contributing and readme guidelines

- Capitalize Cesspool and Slop in contributing guidelines

- Fix markdown table alignment and formatting in style guide