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

## Quickstart

Install Rust 1.75 or later, then:

```bash
cargo install --git https://github.com/YOUR-HANDLE/assay
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
assay verify greet.assay --bundle bundle.xml
```

Exit code is 1 (the stub is not implemented). Run the oracle:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
assay oracle greet.assay --report report.json
```

The oracle emits a directive. Paste the `intent` field into your implementer's prompt. The implementer writes code. Re-bundle. Re-verify. Repeat.

When the report says `"satisfied": true`, you are done.

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

| Command | Purpose |
|---|---|
| `assay verify <spec> --bundle <path>` | Verify a bundle. Emit a report. Exit 0 or 1. |
| `assay report <spec> --bundle <path>` | Same as verify, but always exit 0. |
| `assay diff <report-a> <report-b>` | Compare two reports. |
| `assay oracle <spec> --report <path>` | Invoke the oracle. Emit a directive. |
| `assay loop <spec>` | Run the full loop. |
| `assay init [<dir>]` | Scaffold a new project. |
| `assay fmt <spec>` | Canonicalize a spec. |
| `assay schema` | Print JSON schemas. |
| `assay version` | Print version. |

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

**1. Parse.** It reads a spec and a bundle. A bundle is a single file containing an entire repository, produced by [Repomix](https://github.com/yamadashy/repomix) or a similar packer. Assay understands the Repomix format natively.

**2. Execute.** It extracts the spec's target files from the bundle into a temporary directory. It generates a witness module in the target language. It injects the module into the temporary copy. It invokes the language's own test runner. It captures the output.

**3. Report.** It builds a JSON document listing every witness, its status, the objective score, and a fingerprint over the whole thing. Two runs over the same spec and bundle produce byte-identical reports.

The oracle is a separate step. It reads the report, calls a frontier model, and writes a directive. It never sees the code. It cannot be fooled.

---

## Status

Assay is under active construction. It is being built using its own protocol, which is a bootstrap paradox: to verify Assay, you need Assay.

Current state:

- [x] Crate compiles. CLI recognizes all nine subcommands.
- [x] Canonical types for `Spec`, `Witness`, `Bundle`, `Report`, `Directive`.
- [x] Spec parser implemented.
- [ ] Spec parser tested (the witness suite is next).
- [ ] Bundle parser.
- [ ] Report fingerprinting.
- [ ] Rust runner.
- [ ] Oracle client.
- [ ] Self-hosting: Assay verifying Assay.

Until the loop is self-hosted, verification is performed by hand. See [docs/Day-Zero.md](./docs/Day-Zero.md) for the manual protocol (coming soon).

The roadmap is in [SPEC.md §16](./SPEC.md).

---

## Design Principles

**Verification is deterministic.** No wall-clock time in reports. No environment-dependent behavior. All maps serialized in key order. All floats rounded to a fixed precision. Two runs, two identical outputs.

**Verification is language-agnostic.** Assay does not parse source code. It hands witness bodies to the language's own test runner. Rust, Python, TypeScript, Go, shell. Whatever the target uses.

**Verification is complete over the spec.** If a witness is in the spec, it is checked. There is no way to make a witness disappear from the report without removing it from the spec.

**The oracle is stateless.** It does not maintain its own history. The report contains everything it needs. Its output is a directive; the directive is written to disk; the next invocation reads it.

**The spec is the contract.** The human writes the spec. The implementer writes the code. The two never share a file. The only file they both reference is the bundle, which is generated, not authored.

---

## Security

Assay trusts the spec, the runner configuration, and the host operating system. It does not trust the bundle, the oracle, or the implementer.

- The oracle's output is validated against a JSON schema before being written to disk.
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

---

## Acknowledgments

Inspired by the observation that verification only works when the verifier is simple enough to trust.

Assay is deliberately small. It is small enough that one person can read all of it, understand it, and trust it. That is the point.

The only way to build software that outlives you is to stop writing it.
