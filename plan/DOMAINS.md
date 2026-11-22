# CP Lang domains: one domain = one lane = one owner

Domains are grouped by how much Ryan is involved and how the lane is managed. The segment decides how a lane starts, what Ryan sees, and what counts as evidence.

| Segment | What it is | Ryan's involvement | Evidence | How it runs |
|---|---|---|---|---|
| **A. Service** (main-green, devex, release) | Keep every other lane moving. Reactive work with SLOs; interrupts beat roadmap work. | Almost none: approvals for irreversible ops. | SLOs (red-to-green time, gate time, queue wait, release success), checked continuously. | Every lane starts on its unblocked work at once; A lanes take interrupts first, since every lane depends on them. |
| **B. Metric** (compiler, perf, diagnostics, tech-debt) | The goal is a number, the target is agreed once, then the owner is fully autonomous. | One target sign-off, then a weekly number. Escalate only trade-offs. | The number, from a command, in the goal table; ratchets so it never regresses. | Work toward the drafted target at once; Ryan signs it off asynchronously. No design review. |
| **C. Design, then build** (lint, cli-tools, editor) | The hard part is deciding what to build. One alignment, then the owner builds it alone. | One design review (a 1-2 page design: what, non-goals, acceptance tests), then milestone demos. | The acceptance tests from the design. After the review the lane is a metric lane. | Write the design doc and do the uncontroversial work (audits, fixes, tests) meanwhile; build the new design after Ryan's batched review. |
| **D. Taste and co-design** (spec, stdlib, skill, corpus) | Makes the language feel right. Many judgment calls; the owner proposes, Ryan decides the ones that change the language's character. | Recurring. Bounded by a decision queue, not by chats. | Proxies keep Ryan's load down: token ratio, revision test, dry-run and blind evals, lint false positives. Taste is final. | Owner works ahead on reversible defaults (logged in `decisions.md`) and never blocks on one decision. At most 2-3 of these lanes need Ryan at once. |
| **E. Experience** (site, debugger) | User-facing surfaces: align on a spec and mockups, build, then iterate on what it feels like in use. | Review rounds on live previews, screenshots and recordings. | Browser tests and audits freeze what is agreed; Ryan's review rounds judge feel; the docs-only eval checks the docs. | Every change ships to a preview URL (Worker preview version) with screenshots in the worklog; feedback goes in the domain's feedback list. |

Lanes move between segments. A design-then-build lane becomes a metric lane once its design is approved: the acceptance tests become its number. Decisions from the taste lanes are the inputs the other lanes build against: spec calls become compiler work, idioms become lints. An experience lane ends up as browser tests once the UX settles.

Ryan's time is the scarce resource, not lanes. Segments A and B need almost none, C needs one review per lane (batched), D needs a standing decision queue, E needs review rounds. So every domain runs as a lane from the start, and no lane waits on Ryan: each works on what is unblocked and on reversible defaults, and queues the rest.

Decision queue (D, and C design reviews): `plan/QUESTIONS.md`. One entry per call: the question, options with real code from ports, a recommendation, how to reverse it. Ryan answers in batches. Everything else is an ordinary decision the owner makes and logs in `decisions.md` (as AGENTS.md already says).

Layout: this file is the index. Each domain gets `plan/domains/<domain>/README.md` (covers / improve / success / Ryan's role) and `worklog.md` (newest first: what landed, evidence, next, blocked, requests from other lanes). The previous area map is in `research/design-history/DOMAINS-areas.md`. Two lanes never own the same path.

| # | Seg | Domain | Owns | One line |
|---|---|---|---|---|
| 1 | A | [main-green](domains/main-green/README.md) | CI results, flaky tests, the land queue, reverts | main is green at every commit, and red is fixed in minutes |
| 2 | A | [devex](domains/devex/README.md) | `scripts/` (gate, land, hooks, gen), `justfile`, `mise.toml`, Python tooling, setup docs | ~10 s warm gate, fast landing, one command for everything |
| 3 | A | [release](domains/release/README.md) | `.github/workflows`, `scripts/release`, installers, public-repo hygiene | Install in one line; stable/edge/nightly just work |
| 4 | B | [compiler](domains/compiler/README.md) | `crates/cpl-syntax`, `cpl-check`, `cpl-ir`, `cpl-interp`, `prover` (proofs, range analysis) | The implementation meets the spec; interpreter == C++ |
| 5 | B | [perf](domains/perf/README.md) | `crates/cpl-codegen`, `cpl bench` | C++ as fast and small as hand-written |
| 6 | B | [diagnostics](domains/diagnostics/README.md) | `crates/cpl-diag`, `crates/cpl-fmt`, `spec/diagnostics.md` | One error per root cause, naming the fix; formatter stable on v0.3 |
| 7 | B | [tech-debt](domains/tech-debt/README.md) | crate/file layout, dead code, duplicated helpers, stdlib privilege, 0.2 leftovers | The codebase gets simpler every week |
| 8 | C | [lint](domains/lint/README.md) | `crates/cpl-lint` | Linter rebuilt on the idiom catalog |
| 9 | C | [cli-tools](domains/cli-tools/README.md) | `crates/cpl-cli`, `cpl-contest`, `cpl-doc`, `cpl-json`, `spec/tools/{cli,bundle,explain}.md` | Every `cpl` command is fast, guessable and works on v0.3 |
| 10 | C | [editor](domains/editor/README.md) | `crates/cpl-lsp`, `editors/` | Everything the compiler knows, live in every editor |
| 11 | D | [spec](domains/spec/README.md) | `spec/core`, `spec/lib`, `spec/tools`, `spec/conformance` | What the language is, and tests that catch every compiler bug |
| 12 | D | [stdlib](domains/stdlib/README.md) | `spec/std` (-> `std/`) | Every std body written in CP Lang, documented, tested |
| 13 | D | [skill](domains/skill/README.md) | bundled skill, principles, idiom catalog, `research/dryrun` | A model writes correct, idiomatic CP Lang |
| 14 | D | [corpus](domains/corpus/README.md) | `corpus/` (ports + new problems), `crates/cpl-judge`, scoreboard, `spec/delta/` | 305 idiomatic v0.3 ports proven on every judge; new problems keep exposing gaps |
| 15 | E | [site](domains/site/README.md) | `web/` (content, playground, UI) | A cold reader writes correct code from the docs alone |
| 16 | E | [debugger](domains/debugger/README.md) | `crates/cpl-stress`, `cpl-trace` (split out), `web/packages/trace-viewer` | Doubt to evidence: WA to a minimal counterexample, traps, traces |

Coordinator (me, not a domain): starts and reviews lanes, runs the goal table, batches Ryan's design reviews and decision queue.
