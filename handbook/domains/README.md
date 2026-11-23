# Domains

| | |
|---|---|
| Kind | Process |
| Owner | Coordinator |
| Describes | `handbook/domains/*.md`, `plan/domains/` |

A *domain* is one area of the project with one owner: one body of code, one
feedback loop and one set of concepts, small enough for one agent to hold in
context. Each domain is run by one *lane*, an agent that owns it end to end.
Two domains never own the same path.

## What every domain keeps

| Where | What | Changes |
|---|---|---|
| `handbook/domains/<name>.md` | The charter: mission, scope, interfaces, quality register, suites and tools | When the domain's responsibilities change |
| `plan/domains/<name>/PLAN.md` | What it is working on, in order, toward the current and next milestone | Hourly |
| `plan/domains/<name>/metrics.toml` | Each metric's command, target and current value, run hourly into `plan/STATUS.md` | Every run |
| `plan/domains/<name>/worklog.md` | Newest first: what landed, evidence, next, blocked, requests from other lanes | Every report |

The **quality register** is the heart of a charter. It lists every quality
attribute a user could feel in the domain, each with a metric and the
mechanism that enforces it, or the process used where no mechanism can
decide. How registers work, and how suites are judged, is in
`../process/quality.md`.

## The domains

| Domain | Owns | Excellent means |
|---|---|---|
| [infra](infra.md) | CI, the gate, landing, `scripts/`, release pipeline, installers | Landing takes seconds, `main` recovers fast, installs work everywhere |
| [compiler](compiler.md) | `crates/cpl-syntax`, `cpl-check`, `cpl-ir`, `cpl-interp`, `cpl-prover` | The implementation meets the spec; interpreter and C++ agree |
| [perf](perf.md) | `crates/cpl-codegen`, `cpl bench` | Generated C++ as fast and small as hand-written |
| [diagnostics](diagnostics.md) | `crates/cpl-diag`, `spec/diagnostics.md` | One error per root cause, naming the fix |
| [tech-debt](tech-debt.md) | Cross-domain architecture, ratchets, duplicated and dead code | The codebase gets simpler every week |
| [lint-fmt](lint-fmt.md) | `crates/cpl-lint`, `crates/cpl-fmt` | Lints teach idiomatic code; the formatter is stable |
| [tools](tools.md) | `crates/cpl-cli`, `cpl-contest`, `cpl-doc`, `cpl-json`, `cpl-lsp`, `editors/` | Every command and editor is fast, guessable and works |
| [spec](spec.md) | `spec/`, `handbook/language/`, conformance suite | The language is precise, and its tests catch every compiler bug |
| [stdlib](stdlib.md) | `std/` | A coherent, documented, fast standard library |
| [idioms](idioms.md) | The skill, idiom catalog, `corpus/`, `crates/cpl-judge`, acceptance run | Code written from the docs is correct and idiomatic first time |
| [site](site.md) | `web/` | A cold reader writes correct code from the site alone |
| [debugger](debugger.md) | `crates/cpl-stress`, tracing, the trace viewer | A wrong answer becomes a minimal counterexample in seconds |

## How involved Ryan is

Domains differ in how many judgment calls need Ryan, which decides how a
lane is run:

| Kind | Domains | Ryan's involvement |
|---|---|---|
| Service | infra | Almost none: approves irreversible operations. Interrupts from other lanes beat roadmap work. |
| Metric | compiler, perf, diagnostics, tech-debt | Agrees a target once; the lane then climbs it alone. |
| Design, then build | lint-fmt, tools | One design review, then the lane builds alone and becomes a metric lane. |
| Taste and co-design | spec, stdlib, idioms | The lane proposes; Ryan reviews calls that change the language's character, afterwards, through `plan/QUESTIONS.md`. |
| Experience | site, debugger | Builds, then iterates on how it feels in use, with Ryan's reviews of the live surface. |

Who may change what, and how decisions cross domains, is in
`../process/ownership.md`.

## Changes

- 2026-11-23: first version, from `plan/DOMAINS.md`.
