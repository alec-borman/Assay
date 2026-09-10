# ASSAY

A Universal Verifier and Oracle Protocol for AI-Assisted Software Development.

## Overview

Assay is a program that turns source code into a verifiable mathematical artifact.
It takes a specification and a bundle of code, executes the specification's witnesses, and produces a structured report. It can also invoke an oracle (frontier language model) to produce a directive on how to improve the code.

## Usage

```bash
cargo build --release
./target/release/assay --help
```
