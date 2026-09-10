# ASSAY
## A Universal Verifier and Oracle Protocol for AI-Assisted Software Development

**Version:** 2.0 (Specification)
**License:** MIT OR Apache-2.0
**Status:** Normative

---

## 0. Preface

Assay is a program that turns source code into a verifiable mathematical artifact.

It accepts two inputs:

1. A **spec** — a text file listing witnesses (claims about behavior) that the code must satisfy.
2. A **bundle** — a packed representation of an entire repository (typically produced by Repomix).

It produces one output:

3. A **report** — a deterministic, structured summary of which witnesses pass, which fail, and by how much.

Assay is written in Rust. It shells out to whatever test runner the target language uses. It does not assume the target is Rust, or TypeScript, or Python. It assumes only that the target has some way to run a check and return a boolean.

Assay is intended to sit in the middle of an AI-assisted development loop:

```
          ┌───────────────────────┐
          │  AI Studio Implementer│
          │  (writes code)        │
          └───────────┬───────────┘
                      │ commits
                      ▼
          ┌───────────────────────┐
          │  GitHub Repository    │
          └───────────┬───────────┘
                      │ pull
                      ▼
          ┌───────────────────────┐
          │  Repomix              │
          │  (packs to one file)  │
          └───────────┬───────────┘
                      │ bundle
                      ▼
          ┌───────────────────────┐      ┌──────────────────┐
          │  ASSAY                │─────▶│  report          │
          │  (verifies)           │      └────────┬─────────┘
          └───────────────────────┘               │
                                                  ▼
                                        ┌──────────────────┐
                                        │  Oracle          │
                                        │  (frontier model)│
                                        └────────┬─────────┘
                                                  │ directive
                                                  ▼
                                        back to Implementer
```

The loop has three agents:

- **Implementer** — writes code. Typically an AI Studio agent.
- **Assay** — verifies code against the spec. Deterministic, fast, local.
- **Oracle** — reads Assay's report and issues the next directive. A frontier LLM.

The user (a human) writes the spec and observes the loop. That is the entire system.

This document is self-contained. It assumes no knowledge of any prior system.

---

## 1. Design Principles

### 1.1 Verification is deterministic

Two runs of Assay over the same spec and same bundle produce identical reports. This is enforced by:

- No wall-clock time in the report (only in logs).
- No environment-dependent behavior (locale, timezone, PATH) in report content.
- All maps serialized in key-sorted order.
- All floating-point values rounded to a fixed precision (six decimal places).
- A fingerprint over `{spec_hash, bundle_hash, report_body}` included in every report.

### 1.2 Verification is language-agnostic

Assay does not parse source code. It does not know what language the target is in. It reads a spec, extracts witness definitions written in the target's own language, injects them into a temporary file or runner configuration, and invokes the runner. The runner determines pass/fail.

### 1.3 Verification is complete over the spec

If a witness is in the spec, it is checked. There is no way to make a witness disappear from the report without removing it from the spec. Missing witnesses are failures.

### 1.4 The oracle is stateless

The oracle does not maintain its own history. Assay's report contains everything the oracle needs: current state, delta from the previous state, and the previous directive. The oracle's output is a directive; the directive is written to disk; the next invocation of the oracle reads it.

### 1.5 The spec is the contract

The user writes the spec. The implementer writes the code. The two never share a file. The only file they both reference is the bundle, which is generated, not authored.

---

## 2. Concepts

### 2.1 Spec

A **spec** is a text file. It declares:

- A **target** — which files in the bundle the spec applies to.
- A **runner** — the command that performs the check.
- A set of **witnesses** — claims that must be true.
- A set of **objectives** — soft goals that shape refinement.
- A set of **fixtures** — shared data used by witnesses.

A witness is either a **hard** witness (must pass) or a **soft** witness (contributes to the objective score).

### 2.2 Bundle

A **bundle** is a single file containing the whole repository, produced by a packer such as Repomix. It has:

- A header with metadata.
- A directory listing.
- Each file's content, tagged with its path.

Assay understands the Repomix format natively. Other packers are supported via adapters.

### 2.3 Witness

A **witness** is a claim about the bundle. Formally:

```
witness w = (name, language, expression, expectation)
```

Where:

- `name` — a unique identifier.
- `language` — the language the expression is written in (`rust`, `python`, `typescript`, …).
- `expression` — a snippet of code in that language.
- `expectation` — `pass` if the expression must evaluate to true, `fail` if it must evaluate to false.

Assay does not interpret the expression. It hands it to the runner.

### 2.4 Runner

A **runner** is an external program Assay invokes to execute witnesses. Every runner implements the same contract:

- Accepts a set of witnesses on stdin (or via temp file).
- Returns exit code 0 if all hard witnesses pass, non-zero otherwise.
- Emits a machine-readable summary to stdout.

Assay ships with runners for Rust, Python, Node.js, and a generic shell runner.

### 2.5 Objective

An **objective** is a soft goal. Witnesses marked as `soft` contribute to it. The objective value is a real number in `[0, 1]`:

```
objective = Σ (weightᵢ × passedᵢ) / Σ weightᵢ
```

Where `passedᵢ` is 1 if the soft witness passes, 0 otherwise.

A spec may also declare **measured objectives** — values emitted by the runner as numbers (e.g., "build time in milliseconds"), against which a target and a direction are specified.

### 2.6 Report

A **report** is Assay's output. It is a JSON document. It contains:

- The spec's fingerprint.
- The bundle's fingerprint.
- For each witness: name, status, and on failure, the runner's diagnostic.
- The objective value, if any.
- The delta from the previous report, if a previous report is available.
- The previous directive, echoed back.
- A signed fingerprint over the whole report.

### 2.7 Directive

A **directive** is the oracle's output. It is a JSON document. It contains:

- A target: which file(s) the implementer should modify.
- An intent: a one-sentence description of what the change should accomplish.
- Constraints: witnesses that must continue to pass.
- A rationale: why this directive is the right next step.

### 2.8 Loop

A **loop** is one full cycle:

```
directive → implementer → bundle → assay → report → oracle → directive
```

Assay does not manage the loop. It produces reports. The oracle produces directives. The implementer consumes directives and produces bundles. The loop is the user's to orchestrate, though Assay provides a `assay loop` convenience command.

---

## 3. Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                            ASSAY BINARY                              │
│                                                                      │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────┐             │
│  │  bundle     │    │  spec        │    │  runner      │             │
│  │  parser     │    │  parser      │    │  dispatcher  │             │
│  └──────┬──────┘    └──────┬───────┘    └──────┬───────┘             │
│         │                  │                   │                     │
│         └──────────┬───────┴───────────┬───────┘                     │
│                    ▼                   ▼                             │
│              ┌──────────────┐    ┌──────────────┐                    │
│              │  verifier    │    │  reporter    │                    │
│              │  (core)      │───▶│  (JSON out)  │                    │
│              └──────────────┘    └──────────────┘                    │
│                                                                      │
│              ┌──────────────┐    ┌──────────────┐                    │
│              │  oracle      │    │  loop        │                    │
│              │  client      │    │  orchestrator│                    │
│              └──────────────┘    └──────────────┘                    │
└──────────────────────────────────────────────────────────────────────┘
```

Assay is a single Rust binary. It has no persistent state. All state lives in files the user can see: specs, bundles, reports, directives.

---

## 4. The Spec Format

### 4.1 Syntax

A spec is a plain-text file with the extension `.assay`. It is line-oriented. Leading whitespace is significant only inside witness bodies.

```
# Duration parser — verifies src/duration.rs

spec "duration"

target "src/duration.rs"
runner rust
runners_args ["cargo", "test", "--quiet", "--", "--nocapture"]

lang rust

fixture VALID {
    const VALID: &[(&str, u64)] = &[
        ("1s", 1),
        ("30s", 30),
        ("1m", 60),
        ("2m30s", 150),
    ];
}

witness "empty returns none" hard {
    parse_duration("").is_none()
}

witness "compound minutes-seconds" hard {
    parse_duration("2m30s") == Ok(150)
}

witness "all valid inputs parse" hard {
    VALID.iter().all(|(s, n)| parse_duration(s) == Ok(*n))
}

witness "no panics on garbage" soft weight 0.5 {
    ["", "-", "abc", "1h30"].iter().all(|s| std::panic::catch_unwind(|| parse_duration(s)).is_ok())
}

objective {
    minimises "lines_of_code"
    maximises "test_coverage"
}
```

### 4.2 Directives

| Directive | Meaning |
|---|---|
| `spec "name"` | Names the spec. Required. |
| `target "path"` | The file(s) in the bundle this spec applies to. Glob patterns allowed. Required. |
| `runner <name>` | The runner to use. Required. |
| `runner_args [...]` | Extra arguments passed to the runner. Optional. |
| `lang <name>` | The language of witness bodies. Required. |
| `fixture NAME { ... }` | A named block of language-specific code injected before all witnesses. |
| `witness "name" hard { ... }` | A witness that must pass. |
| `witness "name" soft weight W { ... }` | A witness that contributes `W` to the objective. |
| `objective { ... }` | Declares measured and named objectives. |

### 4.3 Witness Bodies

A witness body is a single language expression enclosed in braces. It may span multiple lines. It must be valid code in the language declared by `lang`.

For Rust, it is a `bool` expression.

For Python, it is an expression that evaluates to a truthy or falsy value.

For TypeScript, it is an expression that evaluates to a `boolean`.

The runner is responsible for interpreting the body in its language.

### 4.4 Fixtures

Fixtures let witnesses share setup code:

```
fixture PARSE_CASES {
    const CASES: &[(&str, u64)] = &[
        ("1s", 1),
        ("30s", 30),
        ("1m", 60),
    ];
}

witness "all cases parse" hard {
    CASES.iter().all(|(s, n)| parse_duration(s) == Ok(*n))
}
```

Fixtures are concatenated in the order declared and injected at the top of the generated test module.

### 4.5 Comments

Lines whose first non-whitespace character is `#` are comments. Inline `#` inside a witness body is a language comment and is preserved.

### 4.6 Full Formal Grammar

```
spec        ::= { line }

line        ::= directive | witness | fixture | objective | comment | blank

directive   ::= spec_decl | target_decl | runner_decl | runner_args_decl | lang_decl
spec_decl   ::= "spec" ws string
target_decl ::= "target" ws string
runner_decl ::= "runner" ws ident
runner_args_decl ::= "runner_args" ws "[" string_list "]"
lang_decl   ::= "lang" ws ident

witness     ::= "witness" ws string ws mode ws "{" nl body nl "}"
mode        ::= "hard" | "soft" ws "weight" ws float

fixture     ::= "fixture" ws ident ws "{" nl body nl "}"

objective   ::= "objective" ws "{" nl { objective_line } "}"
objective_line ::= "minimises" ws string | "maximises" ws string

body        ::= { any_line }

string      ::= '"' { any - '"' } '"'
ident       ::= [a-zA-Z_][a-zA-Z0-9_-]*
float       ::= [0-9]+ ("." [0-9]+)?
ws          ::= " " { " " }
```

---

## 5. Bundle Format

### 5.1 Repomix Format

Assay reads the Repomix XML format natively. A Repomix bundle is:

```xml
<file_summary>
  ...metadata...
</file_summary>

<directory_structure>
  src/
    main.rs
    lib.rs
  ...
</directory_structure>

<files>
  <file path="src/main.rs">
    <![CDATA[
    fn main() { ... }
    ]]>
  </file>
  ...
</files>
```

Assay parses this into an internal `Bundle` struct:

```rust
pub struct Bundle {
    pub files: BTreeMap<String, String>,  // path → content
    pub metadata: BTreeMap<String, String>,
    pub fingerprint: String,              // SHA-256 of canonical serialization
}
```

### 5.2 Bundle Fingerprint

The fingerprint is computed as:

```
for path in sorted(files.keys()):
    hash.update(path)
    hash.update("\0")
    hash.update(files[path])
    hash.update("\0")
fingerprint = hex(hash.finalize())
```

This makes the fingerprint independent of file ordering, metadata, and whitespace around files.

### 5.3 Other Formats

Assay's bundle parser is extensible. The following formats are supported out of the box:

| Format | Detector |
|---|---|
| Repomix XML | Root element `<file_summary>` |
| Repomix Markdown | First line contains `# Repository` |
| Repomix JSON | Root object with `files` array |
| GNU tarball | Magic bytes `\x1f\x8b` |
| Directory | Argument is an existing directory |

Adapters are registered in `src/bundle/`. Each adapter is a small function that takes raw bytes and returns a `Bundle`.

---

## 6. Verification Engine

### 6.1 Sequence

```
1. Parse bundle.
2. Parse spec.
3. For each target file in the spec's glob:
     a. Extract the file from the bundle.
     b. Write it to a temporary directory, preserving path structure.
4. Generate a witness module in the target language.
5. Inject the witness module into the temporary copy of the target file.
6. Write any extra files needed by the runner (e.g., Cargo.toml, package.json).
7. Invoke the runner with the configured arguments.
8. Capture stdout, stderr, and exit code.
9. Parse the runner's output for per-witness status.
10. Build the report.
11. Delete the temporary directory.
12. Emit the report.
```

### 6.2 Runners

Assay ships with the following runners:

| Runner | Language | Invocation |
|---|---|---|
| `rust` | Rust | `cargo test --quiet -- --nocapture` |
| `python` | Python | `pytest -q` or `python -m unittest` |
| `node` | TS/JS | `npx vitest run` or `node --test` |
| `go` | Go | `go test ./...` |
| `shell` | Any | The command named in `runner_args` |

Each runner implements the trait:

```rust
pub trait Runner {
    fn name(&self) -> &'static str;
    fn prepare(&self, dir: &Path, spec: &Spec, witnesses: &[Witness]) -> Result<()>;
    fn invoke(&self, dir: &Path, args: &[String]) -> Result<RunOutput>;
    fn parse(&self, output: &RunOutput, witnesses: &[Witness]) -> Vec<WitnessResult>;
}
```

### 6.3 Runner Output Contract

A runner's output is parsed into `RunOutput`:

```rust
pub struct RunOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub per_witness: Vec<WitnessResult>,  // if the runner can extract them
}

pub struct WitnessResult {
    pub name: String,
    pub status: Status,          // Pass | Fail | Skip | Error
    pub detail: Option<String>,  // stack trace, assertion message, etc.
    pub duration_ms: Option<u64>,
}
```

If the runner cannot extract per-witness results, the whole run is treated as one witness. Runners that can extract per-witness results do so and emit them in a standard format:

```
ASSAY_WITNESS_RESULT name=<name> status=<status> duration_ms=<ms> detail=<single-line detail>
```

Assay's Rust, Python, and Node runners emit this automatically by wrapping the test invocation.

### 6.4 Timeouts

Every runner is invoked with a timeout. Default: 300 seconds. Configurable in `.assay.toml`. On timeout, all witnesses in the run are marked `Error` with detail `"runner timeout"`.

### 6.5 Caching

Assay caches verification results by `(bundle_fingerprint, spec_fingerprint, runner_args)`. Cache lives in `.assay-cache/` at the working directory. Cache entries are invalidated when any component of the key changes.

---

## 7. Report Format

### 7.1 Schema

```json
{
  "assay_version": "2.0.0",
  "spec": {
    "name": "duration",
    "path": "duration.assay",
    "fingerprint": "sha256:...",
    "targets": ["src/duration.rs"],
    "runner": "rust",
    "lang": "rust"
  },
  "bundle": {
    "fingerprint": "sha256:...",
    "file_count": 42,
    "byte_count": 18342
  },
  "run": {
    "timestamp_ms": 1715000000000,
    "duration_ms": 1247,
    "exit_code": 1,
    "cached": false
  },
  "witnesses": [
    {
      "name": "empty returns none",
      "kind": "hard",
      "status": "pass",
      "duration_ms": 3
    },
    {
      "name": "compound minutes-seconds",
      "kind": "hard",
      "status": "fail",
      "duration_ms": 2,
      "detail": "assertion failed: parse_duration(\"2m30s\") == Ok(150)\n  left: Ok(150)\n  right: Ok(150)"
    }
  ],
  "summary": {
    "total": 4,
    "passed": 3,
    "failed": 1,
    "errored": 0,
    "skipped": 0,
    "hard_total": 3,
    "hard_passed": 2,
    "soft_total": 1,
    "soft_passed": 1,
    "satisfied": false,
    "objective": 0.75,
    "objective_breakdown": {
      "no panics on garbage": 0.5
    }
  },
  "delta": {
    "from_fingerprint": "sha256:...",
    "witnesses_changed": [
      {"name": "compound minutes-seconds", "from": "pass", "to": "fail"}
    ],
    "objective_delta": -0.25
  },
  "previous_directive": {
    "path": "directives/latest.json",
    "id": "d-2026-04-03-001",
    "target": "src/duration.rs",
    "intent": "Add support for compound minute-second inputs"
  },
  "report_fingerprint": "sha256:..."
}
```

### 7.2 Fields

| Field | Meaning |
|---|---|
| `spec.fingerprint` | SHA-256 of the spec file's canonical serialization |
| `bundle.fingerprint` | SHA-256 of the bundle (§5.2) |
| `witnesses[].status` | One of `pass`, `fail`, `skip`, `error` |
| `summary.satisfied` | `true` iff `hard_passed == hard_total` and `errored == 0` |
| `summary.objective` | Soft witness score, `[0, 1]` |
| `delta` | Present only if a previous report exists; computed by diffing |
| `previous_directive` | Echoed from disk so the oracle sees continuity |
| `report_fingerprint` | SHA-256 over all fields except itself |

### 7.3 Report Fingerprint

Computed as:

```
body = canonical_json(report without report_fingerprint)
hash = sha256(body)
report_fingerprint = "sha256:" + hex(hash)
```

Canonical JSON: keys sorted, no insignificant whitespace, floats formatted to six decimal places.

### 7.4 Determinism Guarantee

Two invocations of Assay with the same spec and bundle produce byte-identical reports, except for:

- `run.timestamp_ms` and `run.duration_ms` (nondeterministic by nature).
- `delta` (depends on previous report).

To obtain a fully deterministic report, pass `--frozen` to strip these fields. Frozen reports are used for CI caching.

---

## 8. The Oracle Protocol

### 8.1 Role

The oracle is a frontier language model. Its job is to read a report and emit a directive. It does not write code. It does not run tests. It does not see the bundle. It sees only:

- The spec (in full, once per session).
- The current report.
- The previous directive, if any.
- A short history of the last `N` reports and directives, if available.

### 8.2 Inputs

The oracle is invoked with a single JSON object:

```json
{
  "spec": "<full spec text>",
  "report": { ...current report... },
  "history": [
    {"directive": {...}, "report": {...}},
    ...
  ],
  "constraints": {
    "max_targets": 3,
    "max_edit_distance": "moderate",
    "preserve_passing_witnesses": true
  }
}
```

`history` is capped at `oracle.history_limit` entries (default 8).

### 8.3 Output

The oracle emits a single JSON object:

```json
{
  "id": "d-2026-04-03-002",
  "target": ["src/duration.rs"],
  "intent": "Handle 'm' and 's' suffixes in either order.",
  "constraints": [
    "empty returns none",
    "all valid inputs parse"
  ],
  "rationale": "Two hard witnesses fail because the parser assumes a canonical order. Support both orders.",
  "priority": "high",
  "expected_outcome": {
    "witnesses_that_should_flip": ["compound minutes-seconds"],
    "objective_target": 1.0
  }
}
```

### 8.4 Schema

| Field | Type | Meaning |
|---|---|---|
| `id` | string | Unique identifier. Assay generates one if omitted. |
| `target` | array of strings | Paths the implementer may modify. |
| `intent` | string | One sentence describing the change. |
| `constraints` | array of strings | Witness names that must not regress. |
| `rationale` | string | Why this directive is correct. |
| `priority` | string | `high`, `medium`, `low`. |
| `expected_outcome` | object | What should change after applying. |

### 8.5 Invariants

- A directive must target at least one file that appears in the spec's `target` glob.
- A directive must name at least one witness that is currently failing, unless `priority = "high"` and the rationale mentions "refine".
- A directive must not name a witness that is currently passing, unless the rationale mentions "lock in".

These invariants are enforced by Assay when it writes the directive to disk. Invalid directives are rejected with exit code 6.

### 8.6 Persistence

Directives are written to `directives/<id>.json` and `directives/latest.json`. The oracle reads `latest.json` at session start to see the previous directive.

---

## 9. The Refinement Loop

### 9.1 Goal

The loop's goal is twofold:

1. **Satisfaction.** All hard witnesses pass.
2. **Refinement.** The objective value increases monotonically until satisfaction, then plateaus at its maximum.

### 9.2 Loop Steps

```
1. Assay verifies the current bundle.
2. Assay emits a report.
3. Oracle reads the report.
4. Oracle emits a directive.
5. Implementer reads the directive.
6. Implementer modifies the code.
7. Implementer commits.
8. Repository is updated.
9. Bundle is regenerated (via Repomix).
10. Go to 1.
```

### 9.3 Refinement Signals

Assay provides three signals to the oracle:

- **Satisfaction.** Boolean. `true` iff all hard witnesses pass.
- **Objective.** Scalar in `[0, 1]`. Weighted average of soft witnesses.
- **Delta.** Change in satisfaction and objective from the previous report.

### 9.4 Measured Objectives

A spec may declare measured objectives:

```
objective {
    minimises "build_time_ms" target 5000
    maximises "test_coverage_percent" target 95
}
```

The runner emits measured values:

```
ASSAY_MEASURED name=build_time_ms value=4200
ASSAY_MEASURED name=test_coverage_percent value=87.5
```

Assay normalizes each measured value against its target:

```
normalized = clamp(value / target, 0, 1)   for minimises
normalized = clamp(value / target, 0, 1)   for maximises
```

The composite objective is:

```
objective = Σ (wᵢ × softᵢ) + Σ (wⱼ × measuredⱼ) 
            ─────────────────────────────────────
                    Σ wᵢ + Σ wⱼ
```

### 9.5 Convergence Criteria

The loop terminates when any of:

- `satisfied = true` and `objective` has not increased for `K` consecutive iterations (default `K = 3`).
- The user stops it.
- The oracle emits a directive with `intent = "done"`.

### 9.6 `assay loop`

Assay provides a convenience command that orchestrates the loop:

```
assay loop <spec> --bundle <path> --repo <path> --oracle <provider> [--max-iterations N]
```

It:

1. Verifies the bundle.
2. Calls the oracle.
3. Writes the directive.
4. Invokes a user-configured command that applies the directive (typically a script that prompts AI Studio or the implementer).
5. Re-pulls and re-bundles.
6. Repeats.

The user can customize step 4 via `.assay.toml`:

```toml
[loop]
apply_command = "scripts/apply-directive.sh"
repull_command = "scripts/repull.sh"
rebundle_command = "npx repomix --output bundle.xml"
```

Assay does not implement apply, repull, or rebundle itself. It shells out to the user's commands.

---

## 10. CLI Reference

### 10.1 Synopsis

```
assay <command> [options]
```

### 10.2 Commands

| Command | Purpose |
|---|---|
| `assay verify <spec> --bundle <path>` | Verify a bundle against a spec. Emit a report. |
| `assay report <spec> --bundle <path>` | Same as `verify`, but always exit 0. |
| `assay diff <report-a> <report-b>` | Compare two reports. |
| `assay oracle <spec> --report <path>` | Invoke the oracle. Emit a directive. |
| `assay loop <spec>` | Run the loop. |
| `assay init [<dir>]` | Scaffold a new project. |
| `assay fmt <spec>` | Canonicalize a spec. |
| `assay schema` | Print JSON schemas for reports and directives. |
| `assay version` | Print version and exit. |

### 10.3 `verify` Options

| Flag | Meaning |
|---|---|
| `--bundle <path>` | Path to a bundle file. Required. |
| `--out <path>` | Write report to this path. Default: stdout. |
| `--frozen` | Strip nondeterministic fields. |
| `--no-cache` | Skip cache lookup and write. |
| `--runner <name>` | Override the runner declared in the spec. |
| `--target <glob>` | Override the target glob. |
| `--jobs <n>` | Parallelism for multi-target specs. Default: 1. |
| `--quiet` | Suppress logs. |

### 10.4 `oracle` Options

| Flag | Meaning |
|---|---|
| `--report <path>` | Path to a report. Required. |
| `--provider <name>` | `anthropic`, `openai`, `gemini`. Default: auto. |
| `--model <name>` | Model identifier. Default: provider default. |
| `--history <path>` | Directory of prior reports and directives. Default: `directives/`. |
| `--out <path>` | Write directive to this path. Default: `directives/latest.json`. |
| `--max-history <n>` | Max prior entries to include. Default: 8. |

### 10.5 Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success (all hard witnesses pass). |
| 1 | One or more hard witnesses failed. |
| 2 | Spec parse error. |
| 3 | Bundle parse error. |
| 4 | I/O error. |
| 5 | Runner failed to invoke. |
| 6 | Oracle returned an invalid directive. |
| 7 | Timeout. |
| 64 | Invalid CLI usage. |

---

## 11. Configuration

Assay reads `.assay.toml` from the working directory tree, walking upward until found. Environment variables override file values.

```toml
[assay]
working_dir = "."
cache_dir = ".assay-cache"

[verifier]
runner = "auto"
timeout_seconds = 300
max_parallel = 1
emit_measured = true

[bundle]
format = "auto"
max_bytes = 50000000
include = ["src/**", "tests/**", "Cargo.toml", "package.json"]
exclude = ["node_modules/**", "target/**", "dist/**"]

[oracle]
provider = "auto"
model = "auto"
history_limit = 8
max_tokens = 8192
temperature = 0.2
timeout_seconds = 180

[loop]
apply_command = ""
repull_command = ""
rebundle_command = ""
max_iterations = 50
convergence_k = 3

[ui]
color = true
unicode = true
verbosity = "info"
```

### 11.1 Environment Variables

| Variable | Effect |
|---|---|
| `ANTHROPIC_API_KEY` | Enables the Anthropic oracle. |
| `OPENAI_API_KEY` | Enables the OpenAI oracle. |
| `GEMINI_API_KEY` | Enables the Gemini oracle. |
| `ASSAY_CACHE_DIR` | Overrides `assay.cache_dir`. |
| `ASSAY_VERBOSE` | Sets `ui.verbosity = "debug"`. |
| `ASSAY_NO_COLOR` | Sets `ui.color = false`. |

---

## 12. Security Model

### 12.1 Trust Boundaries

Assay trusts:

- The user's spec.
- The user's runner configuration.
- The host operating system.

Assay does not trust:

- The bundle, as authored by arbitrary parties.
- The oracle, as an external language model.
- The implementer, as an autonomous agent.

### 12.2 Filesystem

Assay writes only to:

- The path specified by `--out` (report or directive).
- The cache directory.
- Temporary directories created by `std::env::temp_dir()`, which are deleted on exit.

Assay reads only from:

- The spec path.
- The bundle path.
- The cache directory.

Assay never follows symlinks outside the working directory. Bundle paths containing `..` are rejected.

### 12.3 Network

Assay makes network calls only in `assay oracle` mode, only to the configured provider's API endpoint, and only with the API key present in the environment. API keys are never written to disk, never logged, and never included in any output.

### 12.4 Oracle Output

The oracle's output is treated as untrusted. Assay validates it against the directive schema before writing it. Invalid directives cause exit code 6 and are not written.

### 12.5 Runner Execution

Runners are invoked with the user's permissions in a temporary directory. Assay does not sandbox runners; the user is responsible for the runner's safety. Users running untrusted bundles should invoke Assay inside a container.

---

## 13. Google AI Studio Integration

### 13.1 Overview

The intended deployment is:

1. A user maintains a repository on GitHub.
2. The repository is opened in AI Studio's Build mode.
3. The AI Studio implementer writes code.
4. The user pulls the repository locally.
5. The user runs Assay to produce a report.
6. The user runs the oracle to produce a directive.
7. The directive is fed back into AI Studio as a prompt.
8. Repeat.

Assay automates steps 5–7 with `assay loop`.

### 13.2 Bootstrap Prompt for AI Studio

Paste this into AI Studio Build to generate the initial Assay repository:

> Build a Rust CLI application named `assay`. It reads a specification file and a Repomix bundle, executes the specification's witnesses against the bundle using external runners (Rust, Python, Node.js, and a generic shell runner), and emits a JSON report describing which witnesses pass and which fail. It can also invoke a frontier language model to produce a JSON directive that describes the next change. The application must compile with `cargo build --release` on Rust 1.75+.
>
> Required modules: `bundle`, `spec`, `witness`, `runner`, `report`, `oracle`, `loop`. Required dependencies: `anyhow`, `clap`, `reqwest`, `serde`, `serde_json`, `toml`, `sha2`, `tempfile`. No others.
>
> Provide `assay` subcommands: `verify`, `report`, `diff`, `oracle`, `loop`, `init`, `fmt`, `schema`, `version`. Provide unit tests for the spec parser, the bundle parser, and the report fingerprinting. Provide one integration test that constructs a temporary crate and verifies it end to end. Provide a sample `examples/duration/` with a spec and a stub implementation.

### 13.3 Repository Layout

```
assay/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── .github/workflows/ci.yml
├── src/
│   ├── main.rs
│   ├── bundle/
│   │   ├── mod.rs
│   │   ├── repomix.rs
│   │   ├── markdown.rs
│   │   └── json.rs
│   ├── spec/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── ast.rs
│   ├── witness/
│   │   ├── mod.rs
│   │   └── result.rs
│   ├── runner/
│   │   ├── mod.rs
│   │   ├── rust.rs
│   │   ├── python.rs
│   │   ├── node.rs
│   │   └── shell.rs
│   ├── report/
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   └── fingerprint.rs
│   ├── oracle/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── directive.rs
│   │   └── providers/
│   │       ├── anthropic.rs
│   │       ├── openai.rs
│   │       └── gemini.rs
│   └── loop/
│       ├── mod.rs
│       └── orchestrator.rs
├── examples/
│   └── duration/
│       ├── duration.assay
│       └── src/
│           └── lib.rs
├── tests/
│   ├── spec_parser.rs
│   ├── bundle_parser.rs
│   ├── report_fingerprint.rs
│   └── end_to_end.rs
└── docs/
    ├── spec.md
    ├── grammar.ebnf
    ├── report.schema.json
    └── directive.schema.json
```

---

## 14. Complete Example

### 14.1 The Spec

`examples/duration/duration.assay`:

```
spec "duration"

target "src/lib.rs"
runner rust
lang rust

fixture VALID {
    const VALID: &[(&str, u64)] = &[
        ("1s", 1),
        ("30s", 30),
        ("1m", 60),
        ("2m30s", 150),
        ("1h", 3600),
        ("1h30m", 5400),
    ];
}

fixture INVALID {
    const INVALID: &[&str] = &[
        "",
        "s",
        "1",
        "1x",
        "-1s",
        "1h30",
    ];
}

witness "empty input" hard {
    parse_duration("").is_none()
}

witness "single second" hard {
    parse_duration("1s") == Ok(1)
}

witness "compound minutes-seconds" hard {
    parse_duration("2m30s") == Ok(150)
}

witness "compound hours-minutes" hard {
    parse_duration("1h30m") == Ok(5400)
}

witness "all valid inputs parse" hard {
    VALID.iter().all(|(s, n)| parse_duration(s) == Ok(*n))
}

witness "all invalid inputs reject" hard {
    INVALID.iter().all(|s| parse_duration(s).is_err())
}

witness "no panics on arbitrary input" soft weight 1.0 {
    [
        "\u{0000}", "１２３", "1s1s", "9999999999999999999s",
    ].iter().all(|s| parse_duration(s).is_err())
}

objective {
    minimises "lines_of_code" target 200
}
```

### 14.2 The Implementation Stub

`examples/duration/src/lib.rs`:

```rust
pub fn parse_duration(_s: &str) -> Result<u64, String> {
    unimplemented!()
}
```

### 14.3 Verification

```bash
$ assay verify examples/duration/duration.assay --bundle examples/duration/bundle.xml
{
  "assay_version": "2.0.0",
  "spec": { "name": "duration", ... },
  "summary": {
    "total": 7,
    "passed": 0,
    "failed": 6,
    "errored": 0,
    "hard_total": 6,
    "hard_passed": 0,
    "soft_total": 1,
    "soft_passed": 0,
    "satisfied": false,
    "objective": 0.0
  },
  ...
}
```

Exit code 1.

### 14.4 Oracle Invocation

```bash
$ assay oracle examples/duration/duration.assay --report report.json
{
  "id": "d-2026-04-03-001",
  "target": ["src/lib.rs"],
  "intent": "Implement parse_duration to parse sequences of numeric-amount+unit-suffix pairs (s for seconds, m for minutes, h for hours), summing their values.",
  "constraints": [],
  "rationale": "All six hard witnesses concern parse_duration. The stub is unimplemented. The simplest implementation parses the string left to right, extracting (amount, unit) pairs and summing.",
  "priority": "high",
  "expected_outcome": {
    "witnesses_that_should_flip": [
      "empty input",
      "single second",
      "compound minutes-seconds",
      "compound hours-minutes",
      "all valid inputs parse",
      "all invalid inputs reject"
    ],
    "objective_target": 1.0
  }
}
```

### 14.5 Applying the Directive

The user pastes the `intent` field into AI Studio. The implementer writes:

```rust
pub fn parse_duration(s: &str) -> Result<u64, String> {
    if s.is_empty() { return Err("empty".into()); }
    let mut total = 0u64;
    let mut num = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let n: u64 = num.parse().map_err(|_| "no number")?;
            let mult = match c {
                's' => 1,
                'm' => 60,
                'h' => 3600,
                _ => return Err("bad unit".into()),
            };
            total += n * mult;
            num.clear();
        }
    }
    if !num.is_empty() { return Err("trailing number".into()); }
    Ok(total)
}
```

### 14.6 Re-Verification

```bash
$ assay verify examples/duration/duration.assay --bundle examples/duration/bundle.xml
{
  ...
  "summary": {
    "satisfied": true,
    "objective": 1.0,
    ...
  }
}
```

Exit code 0. The loop completes.

---

## 15. Non-Goals

Assay is not:

- A build system. Use `cargo`, `npm`, `make`, or whatever the target uses.
- A package manager. Use the target's native tooling.
- A version control system. Use `git`.
- A code editor. Use AI Studio or an IDE.
- A linter. Use the target's linter.
- A formatter. Use the target's formatter.
- A code reviewer. Reviews are for humans.
- A specification language. Specs are witnesses; they are written in the target's language.
- A test framework. Runners are test frameworks. Assay invokes them.
- A scheduler. Loops are the user's to orchestrate.

Assay does one thing: it takes a spec and a bundle, executes the spec's witnesses against the bundle, and emits a structured report. Everything else is downstream.

---

## 16. Roadmap

**v2.0** — This document.

**v2.1** — Additional runners: Go, Java, Ruby, PHP, C#. Bundle adapters for `git archive`, `tar`, `zip`, and plain directories. Cache compression.

**v2.2** — Parallel target verification. Per-witness caching (finer grain than per-run caching). Report diffing with structural awareness (identifies which witnesses changed for which reasons).

**v2.3** — Incremental verification: skip witnesses whose dependencies did not change. Requires a per-witness dependency declaration.

**v2.4** — Oracle session memory: Assay keeps a rolling summary of the last `N` directives and reports, and provides it to the oracle as a compressed context. This reduces token usage and improves directive coherence.

**v3.0** — Verifiable refinement: witnesses may declare formal predicates over the objective function (e.g., "this witness can only improve"). Assay tracks which directives moved the objective and which did not, and shapes future oracle prompts accordingly.

Everything after v3.0 is speculative and will be driven by real use.

---

## 17. Glossary

**Assay.** The program described by this document.

**Bundle.** A single file containing an entire repository. Typically produced by Repomix.

**Delta.** The change between two reports. Present in a report only if a previous report is available.

**Directive.** The oracle's output. A JSON document describing the next change.

**Fingerprint.** A SHA-256 hash over a canonical serialization. Used to identify specs, bundles, and reports.

**Fixture.** A named block of language-specific code injected into a generated witness module.

**Hard witness.** A witness that must pass for the spec to be considered satisfied.

**Implementer.** The agent that writes code. Typically AI Studio.

**Loop.** One full cycle: directive → implementer → bundle → assay → report → oracle.

**Objective.** A scalar in `[0, 1]` measuring soft witness satisfaction and measured objectives.

**Oracle.** The frontier language model that reads reports and emits directives.

**Report.** Assay's output. A JSON document describing witness statuses and the objective.

**Runner.** An external program that Assay invokes to execute witnesses in a specific language.

**Soft witness.** A witness that contributes to the objective but does not gate satisfaction.

**Spec.** A text file declaring targets, runners, witnesses, fixtures, and objectives.

**Satisfaction.** `true` iff every hard witness passes.

**Witness.** A claim about the bundle. Compiled into a test in the target language.

---

## 18. Final Note

The one-sentence description of Assay is:

> **Given a spec and a bundle, execute the spec's witnesses against the bundle, and report the result deterministically.**

The oracle's one-sentence description is:

> **Given a report, produce a directive that moves the code toward satisfaction and refinement.**

The loop's one-sentence description is:

> **Turn the code into a verifiable math problem, and solve it by iterating with an oracle and an implementer.**

Everything else in this document is detail. If the implementation of Assay grows beyond what is needed to support these three sentences, it is wrong.

Assay is deliberately small. It is small enough that one person can read all of it, understand it, and trust it. That is the point. Verification only works when the verifier is simple enough to trust.

— *End of Specification*