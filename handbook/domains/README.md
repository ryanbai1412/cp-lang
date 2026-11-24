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

Every charter also carries a standing slice of about 5% of the lane's time
for hunting debt that no counter sees: a second mechanism, a list with no
counter, a workaround someone routed around. A find gets a ratchet counter
the day it is found, under the rules in
[architecture invariants](../design/architecture-invariants.md).

The **quality register** is the heart of a charter. It lists every quality
attribute a user could feel in the domain, each with a metric and the
mechanism that enforces it, or the process used where no mechanism can
decide. How registers work, and how suites are judged, is in
`../process/quality.md`.

## The domains

| Domain | Excellent means |
|---|---|
| [infra](infra.md) | Landing takes seconds, `main` recovers fast, installs work everywhere |
| [compiler](compiler.md) | The implementation meets the spec; interpreter and C++ agree |
| [perf](perf.md) | Generated C++ as fast and small as hand-written |
| [diagnostics](diagnostics.md) | One error per root cause, naming the fix |
| [tech-debt](tech-debt.md) | The codebase gets simpler every week |
| [lint-fmt](lint-fmt.md) | Lints teach idiomatic code; the formatter is stable |
| [tools](tools.md) | Every command and editor is fast, guessable and works |
| [spec](spec.md) | The language is precise, and its tests catch every compiler bug |
| [stdlib](stdlib.md) | A coherent, documented, fast standard library |
| [idioms](idioms.md) | Code written from the docs is correct and idiomatic first time |
| [site](site.md) | A cold reader writes correct code from the site alone |
| [debugger](debugger.md) | A wrong answer becomes a minimal counterexample in seconds |

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
