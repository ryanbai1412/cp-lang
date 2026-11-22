# CP Lang domains: one domain = one lane = one owner

Domains are grouped by how much Ryan is involved and how the lane is managed. The segment decides how a lane starts, what Ryan sees, and what counts as evidence.

| Segment | What it is | Ryan's involvement | Evidence | How it runs |
|---|---|---|---|---|
| **A. Service** (infra) | Keep every other lane moving. Reactive work with SLOs; interrupts beat roadmap work. | Almost none: approvals for irreversible ops. | SLOs (red-to-green time, gate time, queue wait, release success), checked continuously. | Every lane starts on its unblocked work at once; A lanes take interrupts first, since every lane depends on them. |
| **B. Metric** (compiler, perf, diagnostics, architecture & tech-debt) | The goal is a number, the target is agreed once, then the owner is fully autonomous. | One target sign-off, then a weekly number. Escalate only trade-offs. | The number, from a command, in the goal table; ratchets so it never regresses. | Work toward the drafted target at once; Ryan signs it off asynchronously. No design review. |
| **C. Design, then build** (lint-fmt, tools) | The hard part is deciding what to build. One alignment, then the owner builds it alone. | One design review (a 1-2 page design: what, non-goals, acceptance tests), then milestone demos. | The acceptance tests from the design. After the review the lane is a metric lane. | Write the design doc and do the uncontroversial work (audits, fixes, tests) meanwhile; build the new design after Ryan's batched review. |
| **D. Taste and co-design** (spec, stdlib, idioms) | Makes the language feel right. Many judgment calls; the owner proposes, Ryan decides the ones that change the language's character. | Recurring. Bounded by a decision queue, not by chats. | Proxies keep Ryan's load down: token ratio, revision test, dry-run and blind evals, lint false positives. Taste is final. | Owner works ahead on reversible defaults (logged in `decisions.md`) and never blocks on one decision. At most 2-3 of these lanes need Ryan at once. |
| **E. Experience** (site, debugger) | User-facing surfaces: align on a spec and mockups, build, then iterate on what it feels like in use. | Review rounds on live previews, screenshots and recordings. | Browser tests and audits freeze what is agreed; Ryan's review rounds judge feel; the docs-only eval checks the docs. | Every change ships to a preview URL (Worker preview version) with screenshots in the worklog; feedback goes in the domain's feedback list. |

Lanes move between segments. A design-then-build lane becomes a metric lane once its design is approved: the acceptance tests become its number. Decisions from the taste lanes are the inputs the other lanes build against: spec calls become compiler work, idioms become lints. An experience lane ends up as browser tests once the UX settles.

Ryan's time is the scarce resource, not lanes. Segments A and B need almost none, C needs one review per lane (batched), D needs a standing decision queue, E needs review rounds. So every domain runs as a lane from the start, and no lane waits on Ryan: each works on what is unblocked and on reversible defaults, and queues the rest.

Decision queue (D, and C design reviews): `plan/QUESTIONS.md`. One entry per call: the question, options with real code from ports, a recommendation, how to reverse it. Ryan answers in batches. Everything else is an ordinary decision the owner makes and logs in `decisions.md` (as AGENTS.md already says).

Layout: this file is the index. Every domain uses the same layout, copied from `plan/domains/TEMPLATE/`: `README.md` (one page Ryan reviews: mission, covers, success criteria, 2-3 milestones with checkable exit criteria, metrics, judgment rubric and probes, decisions for Ryan, status), `PLAN.md` (workstreams, order, alternatives, risks), `metrics.toml` (each metric's command, target and value; run hourly), `ARCHITECTURE.md` (the code as it is: modules, interfaces other domains use and their consumers, invariants, known debt; big domains add `architecture/<topic>.md`), optional short-lived `design/<topic>.md` (only what `spec/` does not cover; folded into `spec/` or deleted once built), and `worklog.md` (newest first: what landed, evidence, next, blocked, requests from other lanes). `plan/ARCHITECTURE.md` is the cross-domain map, owned by architecture & tech-debt. The previous area map is in `research/design-history/DOMAINS-areas.md`. Two lanes never own the same path.

| # | Seg | Domain | Owns | One line |
|---|---|---|---|---|
| 1 | A | [infra](domains/infra/README.md) | CI, gate, land queue, `scripts/`, `justfile`, `mise.toml`, setup docs, release pipeline, installers, public-repo hygiene | main green at every commit, ~10 s warm gate, install in one line |
| 2 | B | [compiler](domains/compiler/README.md) | `crates/cpl-syntax`, `cpl-check`, `cpl-ir`, `cpl-interp`, `prover`; the shared core interfaces | The implementation meets the spec; interpreter == C++ |
| 3 | B | [perf](domains/perf/README.md) | `crates/cpl-codegen`, `cpl bench` | C++ as fast and small as hand-written |
| 4 | B | [diagnostics](domains/diagnostics/README.md) | `crates/cpl-diag`, `spec/diagnostics.md` | One error per root cause, naming the fix |
| 5 | B | [architecture & tech-debt](domains/tech-debt/README.md) | `plan/ARCHITECTURE.md`, crate/file layout, dead code, duplicated helpers, stdlib privilege, 0.2 leftovers | The codebase gets simpler every week; clean architecture across lanes |
| 6 | C | [lint-fmt](domains/lint-fmt/README.md) | `crates/cpl-lint`, `crates/cpl-fmt` | Lints from the idiom catalog; formatter stable on v0.3 |
| 7 | C | [tools](domains/tools/README.md) | `crates/cpl-cli`, `cpl-contest`, `cpl-doc`, `cpl-json`, `cpl-lsp`, `editors/` | Every command and editor fast, guessable, working on v0.3 |
| 8 | D | [spec](domains/spec/README.md) | `spec/core`, `spec/lib`, `spec/tools`, `spec/conformance`, `crates/cpl-conform`, `scripts/conformance-*.py` | What the language is, and tests that catch every compiler bug |
| 9 | D | [stdlib](domains/stdlib/README.md) | `spec/std` (-> `std/`) | Every std body written in CP Lang, documented, tested |
| 10 | D | [idioms](domains/idioms/README.md) | skill, principles, idiom catalog, `research/dryrun`, `corpus/`, `crates/cpl-judge`, scoreboard | A model writes correct, idiomatic CP Lang; 305 ports proven on every judge |
| 11 | E | [site](domains/site/README.md) | `web/` (content, playground, UI) | A cold reader writes correct code from the docs alone |
| 12 | E | [debugger](domains/debugger/README.md) | `crates/cpl-stress`, `cpl-trace` (split out), `web/packages/trace-viewer` | Doubt to evidence: WA to a minimal counterexample, traps, traces |

Lane boundaries follow what one agent can hold in context: one body of code, one feedback loop, one set of concepts.

## Keeping the architecture clean

- **Ratchets in the gate**: file size, duplicated helpers, the crate dependency graph (a new edge needs a `plan/ARCHITECTURE.md` change in the same commit), `#[ignore]`s, builtins and special cases. A lane cannot land a change that raises one.
- **Architecture first**: a change that adds a module, a crate edge or a concept updates the domain's `ARCHITECTURE.md` (and the cross-domain map when it crosses domains) in the same commit, with a short design note in the worklog.
- **The shared core is a set of interfaces with named consumers** (`domains/compiler/ARCHITECTURE.md`). Consumers add contract tests next to the core; they ask for new interfaces in the compiler worklog instead of working around a gap; architecture & tech-debt reviews every interface change against all its consumers.
- **Cold reviews of landed code**: architecture & tech-debt reviews each lane's recent diffs on a rotation against a fixed rubric; high-impact findings go in the owning lane's worklog and come before its new feature work.
- **Change-cost probes**: a fixed set of everyday changes done by a cold agent; files touched and time must trend down.
- The coordinator reads a sample of every lane's diffs each round.

Coordinator (me, not a domain): starts and reviews lanes, runs the goal table, batches Ryan's design reviews and decision queue.
