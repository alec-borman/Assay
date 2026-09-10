Here is the updated README, accurate to the current state of the repo. Same voice, same structure, only the facts changed.

---

# Assay

**The verifier for AI-written code.**

Assay turns any source repository into a verifiable artifact. You write a spec. Assay executes the spec's witnesses against the code. It emits a deterministic report. A frontier model reads the report and issues a directive. An implementer applies the directive. Loop until satisfied.

That is the entire system.

```
spec + bundle  ──▶  Assay  ──▶  report  ──▶  Oracle  ──▶  directive  ──▶  Implementer
                     ▲                                                          │
                     └──────────────────────────────────────────────────────────┘
```

---

## Why

LLMs write code that looks right. It compiles. It passes the tests the model also wrote. It ships. Then it fails on the case nobody checked.

The problem is not that AI writes bad code. It is that AI writes *plausible* code, and plausibility is not correctness. There is no machine in the loop that decides whether the code actually satisfies what you asked for.

Assay is that machine.

- The **spec** is written by a human. It is a plain text file of assertions.
- The **report** is written by Assay. It is deterministic and inspectable.
- The **directive** is written by a frontier model that cannot see the code, only the report.
- The **implementation** is written by an AI that cannot hide failure, because the compiler is the arbiter.

The unit of progress is a verified witness. Not a merged PR. Not a code review. A claim that provably holds.

---

## Day Zero

Assay verifies Assay.

```
$ cargo run -- verify assay.assay --bundle repomix-output.xml
{
  "spec":   { "name": "assay", "fingerprint": "sha256:db710f10..." },
  "bundle": { "file_count": 59, "byte_count": 175353,
              "fingerprint": "sha256:ea08d00c..." },
  "summary": { "total": 6, "passed": 6, "failed": 0,
               "hard_total": 6, "hard_passed": 6,
               "satisfied": true },
  "report_fingerprint": "sha256:7eca0a86..."
}
```

Exit code 0. That report was produced by an actual `cargo test` run inside a temp crate that Assay built and populated from its own source, extracted from a Repomix bundle. Nothing stubbed. Nothing mocked. Six witnesses, six passes.

The bootstrap problem is now closed. What remains is the oracle client and the loop driver, which together retire the manual protocol currently used to run the loop by hand.

---

## Quickstart

Install Rust 1.75 or later. Clone the repository and build:

```bash
git clone https://github.com/YOUR-HANDLE/assay
cd assay
cargo build --release
```

Write a spec. Here is the smallest possible one:

```
spec "greet"

target "src/lib.rs"
runner rust
lang rust

witness "greets by name" hard {
    greet("world") == "hello, world"
}
```

Write a stub in `src/lib.rs`:

```rust
pub fn greet(_name: &str) -> String {
    unimplemented!()
}
```

Pack the repository into a bundle:

```bash
npx repomix
```

Verify:

```bash
assay verify greet.assay --bundle repomix-output.xml
```

Exit code is 1. The stub is not implemented.

To produce a report regardless of satisfaction, pass `--out`:

```bash
assay verify greet.assay --bundle repomix-output.xml --out report.json
```

The oracle client is not yet implemented. Until it is, the loop is run by hand: read the report, write the next directive yourself, hand it to the implementer, re-bundle, re-verify. See [docs/spec.md](./docs/spec.md) for the full protocol.

---

## The Spec Format

Assay specs are line-oriented and human-readable. A spec has directives, witnesses, fixtures, and objectives.

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
    ];
}

witness "empty input rejects" hard {
    parse_duration("").is_none()
}

witness "single second parses" hard {
    parse_duration("1s") == Ok(1)
}

witness "compound minutes and seconds" hard {
    parse_duration("2m30s") == Ok(150)
}

witness "all valid inputs parse" hard {
    VALID.iter().all(|(s, n)| parse_duration(s) == Ok(*n))
}

witness "no panics on garbage" soft weight 1.0 {
    ["", "-", "abc"].iter().all(|s| std::panic::catch_unwind(|| parse_duration(s)).is_ok())
}

objective {
    minimises "lines_of_code" target 200
}
```

A **hard witness** must pass for the spec to be satisfied.
A **soft witness** contributes to the objective score but does not gate satisfaction.
A **fixture** is shared code injected before every witness.
An **objective** declares soft goals and measured targets.

Full grammar and semantics are in [SPEC.md](./SPEC.md).

---

## Commands

| Command | Purpose | Status |
|---|---|---|
| `assay verify <spec> --bundle <path>` | Verify a bundle. Emit a report. Exit 0 or 1. | implemented |
| `assay report <spec> --bundle <path>` | Same as verify, but always exit 0. | stub |
| `assay diff <report-a> <report-b>` | Compare two reports. | stub |
| `assay oracle <spec> --report <path>` | Invoke the oracle. Emit a directive. | not implemented |
| `assay loop <spec>` | Run the full loop. | not implemented |
| `assay init [<dir>]` | Scaffold a new project. | stub |
| `assay fmt <spec>` | Canonicalize a spec. | stub |
| `assay schema` | Print JSON schemas. | stub |
| `assay version` | Print version. | implemented |

Exit codes:

| Code | Meaning |
|---|---|
| 0 | All hard witnesses pass. |
| 1 | One or more hard witnesses failed. |
| 2 | Spec parse error. |
| 3 | Bundle parse error. |
| 4 | I/O error. |
| 5 | Runner failed to invoke. |
| 6 | Oracle returned an invalid directive. |
| 7 | Timeout. |
| 64 | Invalid CLI usage. |

---

## How It Works

Assay has three jobs and nothing else.

**1. Parse.** It reads a spec and a bundle. A bundle is a single file containing an entire repository, produced by [Repomix](https://github.com/yamadashy/repomix) or a similar packer. Assay understands the Repomix format natively, including CDATA escaping.

**2. Execute.** It extracts the spec's target files from the bundle into a temporary directory. It generates a witness module in the target language. It injects the module into the temporary copy. It invokes the language's own test runner. It captures the output.

**3. Report.** It builds a JSON document listing every witness, its status, the objective score, and a fingerprint over the whole thing. Two runs over the same spec and bundle produce byte-identical reports, except for timestamps.

The oracle is a separate step. It reads the report, calls a frontier model, and writes a directive. It never sees the code. It cannot be fooled.

---

## Status

Assay is under active construction, and it is being built using its own protocol. Day Zero has been reached: Assay verifies Assay.

**Working:**

- [x] Crate compiles. CLI recognizes all nine subcommands.
- [x] Canonical types for `Spec`, `Witness`, `Bundle`, `Report`, `Directive`.
- [x] Spec parser, with tests.
- [x] Bundle parser (Repomix XML, CDATA-aware), with tests.
- [x] Report builder and canonical JSON fingerprint, with tests.
- [x] Rust runner (extract, inject, invoke, parse), with tests.
- [x] `verify` orchestrator and CLI wiring.
- [x] 25 tests green.
- [x] Self-hosting: Assay verifies Assay, 6 of 6 witnesses.

**Not yet implemented:**

- [ ] Oracle client (`src/oracle/client.rs` and the three providers).
- [ ] Loop orchestrator (`src/orchestration/orchestrator.rs`).
- [ ] Python, Node, and Shell runners (currently stubs).
- [ ] Bundle adapters for Markdown, JSON, tarball, and directory.
- [ ] Cache, `.assay.toml`, timeouts, report delta, `--frozen` strips.
- [ ] The `report`, `diff`, `init`, `fmt`, and `schema` subcommands.

Until the oracle client and loop driver land, the loop is run by hand. The manual protocol is documented in [docs/Assay_Lite.md](./docs/Assay_Lite.md).

The roadmap is in [SPEC.md §16](./SPEC.md).

---

## Design Principles

**Verification is deterministic.** No wall-clock time in the report body. No environment-dependent behavior. All maps serialized in key order. Two runs, two identical reports.

**Verification is language-agnostic.** Assay does not parse source code. It hands witness bodies to the language's own test runner. Rust, Python, TypeScript, Go, shell. Whatever the target uses.

**Verification is complete over the spec.** If a witness is in the spec, it is checked. There is no way to make a witness disappear from the report without removing it from the spec.

**The oracle is stateless.** It does not maintain its own history. The report contains everything it needs. Its output is a directive; the directive is written to disk; the next invocation reads it.

**The spec is the contract.** The human writes the spec. The implementer writes the code. The two never share a file. The only file they both reference is the bundle, which is generated, not authored.

---

## Security

Assay trusts the spec, the runner configuration, and the host operating system. It does not trust the bundle, the oracle, or the implementer.

- The oracle's output will be validated against a JSON schema before being written to disk. This is not yet implemented.
- API keys are read from the environment only. They are never written to disk, never logged, and never included in any output.
- Assay writes only to the paths specified by `--out`, the cache directory, and a temporary directory that is deleted on exit.
- Bundle paths containing `..` are rejected.
- Runners execute with the user's permissions. Users running untrusted bundles should invoke Assay inside a container.

Details in [SPEC.md §12](./SPEC.md).

---

## What Assay Is Not

Assay is not a build system. It is not a package manager. It is not a linter. It is not a formatter. It is not a code reviewer. It is not a test framework. It is not a scheduler.

It is a verifier. It takes a spec and a bundle, executes the spec's witnesses against the bundle, and reports the result deterministically. Everything else is downstream.

---

## Contributing

Assay is small on purpose. Before adding anything, ask whether the addition supports this sentence:

> Given a spec and a bundle, execute the spec's witnesses against the bundle, and report the result deterministically.

If it does not, it belongs in a different program.

Contributions welcome. Open an issue before opening a PR on anything larger than a typo.

---

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))
- MIT License ([LICENSE-MIT](./LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

Inspired by the observation that verification only works when the verifier is simple enough to trust.

Assay is deliberately small. It is small enough that one person can read all of it, understand it, and trust it. That is the point.
