# Virtual Assay

Two chatbots. One spec. The human carries messages between them. No cargo, no bundle, no parser. What's left is the discipline — and the discipline is most of the value.

---

## The two roles

**Oracle** — reads code, produces reports and directives. Does not write code. Does not trust the Implementer's self-reports. Reads the actual files.

**Implementer** — writes code. Receives directives. Never sees the spec, only the current directive.

The human is the transport layer. Every message goes Oracle → human → Implementer → human → Oracle.

---

## The shared file: `spec.md`

Ten witnesses, no more. Each one is a sentence and a check. Written by the human. Neither chatbot writes it.

Example:

```
# spec: duration parser

W1. parse_duration("") is Err
W2. parse_duration("1s") == Ok(1)
W3. parse_duration("2m30s") == Ok(150)
W4. parse_duration("1h30m") == Ok(5400)
W5. parse_duration("abc") is Err
W6. parse_duration("1") is Err
W7. parse_duration("-1s") is Err
W8. parse_duration("1x") is Err
W9. no panics on any string
W10. signature is exactly `fn parse_duration(&str) -> Result<u64, String>`
```

Ten is a cap. If you want an eleventh, remove one.

---

## The report format

The Oracle produces this after reading the current code. It is a prediction, not a measurement. It says so.

```
REPORT 003
status: unverified (oracle simulation)

W1  pass
W2  pass
W3  fail  — line 14 parses "30s" as 30 but ignores the "m" branch
W4  fail  — same
W5  pass
W6  pass
W7  fail  — no sign handling; "-1s" would panic on parse().unwrap()
W8  pass
W9  fail  — see W7
W10 pass

satisfied: false
failing: W3, W4, W7, W9
next: fix parsing loop to handle multiple (number, unit) pairs
```

Every line is grounded in a line number and a claim about that line. If the Oracle cannot point to the line, it writes `unknown` and the human goes back and asks.

---

## The directive format

```
DIRECTIVE 003
target: src/lib.rs
intent: parse left-to-right, accumulating (number, unit) pairs;
        reject the string if any residue remains
constraints:
  - W1, W2, W5, W6, W8 must still pass
do not touch:
  - Cargo.toml
  - the public signature
expected: W3, W4, W7, W9 flip to pass
```

One file. One intent. One or two constraints. One list of witnesses that must not regress.

---

## The loop

```
1. Human writes spec.md           (once, at the start)
2. Implementer writes the code    (a fresh window; paste the file paths)
3. Human pastes code into Oracle  (paste the actual file contents)
4. Oracle produces REPORT N
5. Oracle produces DIRECTIVE N+1
6. Human pastes DIRECTIVE N+1 into Implementer
7. Implementer writes the code
8. Repeat from 3
```

The Oracle always produces both. The Implementer never sees the spec or the report.

---

## The three rules that make it work

**The Oracle reads code, not summaries.** Not "the implementer says X". Not "here is a diff". The whole file, every iteration. The whole point is that the Oracle is judging the artifact, not the description of the artifact.

**The Implementer never sees the spec.** If the Implementer sees W3, it will inline `"2m30s" → 150` and call it done. The Implementer gets the *intent* — "parse left-to-right" — not the *check*. That is the whole trick.

**If the Oracle cannot name the line, the witness is `unknown`.** Not pass, not fail. `unknown`. And the human goes back and asks the Oracle to re-read the specific region. No guessing.

---

## What this loses

- No actual execution. A witness like W9 ("no panics on any string") is a hard promise to keep honestly. The Oracle will sometimes be wrong.
- No fingerprint, no determinism, no caching, no directive invariants, no confidence intervals.
- It does not scale to 100 witnesses. Ten is the ceiling.

## What this keeps

- The loop shape. Spec → implement → judge → directive → implement.
- The separation of roles. The Implementer cannot see the checks.
- The single-file, single-intent directive.
- The report as a machine-readable artifact, even if the machine is a model.
- The "must not regress" constraint.

That's most of what makes Assay work when it works.

---

## Two system prompts

Paste this into the **Oracle** window:

> You are the Oracle for a software project. You read source files and produce reports and directives. You do not write code. You do not trust descriptions of the code — you read the code itself. Your output format is fixed:
>
> REPORT N
> status: unverified (oracle simulation)
> [one line per witness: pass / fail / unknown, plus a line number and a claim for every fail]
> satisfied: true|false
> failing: [list]
> next: [one sentence]
>
> DIRECTIVE N
> target: [one file]
> intent: [one sentence]
> constraints: [witnesses that must not regress]
> do not touch: [files or regions]
> expected: [witnesses that should flip]
>
> If you cannot point to a specific line for a witness's status, write `unknown`. Never guess pass or fail.

Paste this into the **Implementer** window:

> You are the Implementer for a software project. You receive a single directive at a time. You write code to satisfy the intent, and you must not break the listed constraints. You never see the spec or the tests. Do not add tests. Do not add comments explaining yourself. Return the full contents of the target file, not a diff. If the directive names a file you cannot see, ask for it. If the directive is ambiguous, say so; do not guess.

---

That's it. Two prompts, one spec, one loop, three rules. It is not Assay, but it is the shape of Assay, and it will catch most of the things Assay would have caught.
