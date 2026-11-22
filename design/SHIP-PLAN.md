# Shipping the new design in cp-lang [proposed, 2026-11-21]

Ryan, 2026-11-21: "take over and own cp-lang and just ship all this syntax that we currently have planned."

**Starting point (measured 2026-11-21).** `ryanbai1412/cp-lang` at `3869a20`: CP Lang 0.2.2, a 105k-line Rust
workspace (parser, checker with lexical `lends`, typed IR, interpreter, C++17 codegen, fmt, lint, stress, LSP,
wasm playground), 6.3k lines of stdlib (25 modules, 322 public items), 370 files in `proto/ports`, 579 corpus
problems ported, and `cargo test --release` fully green. No hosted CI: `scripts/check.sh --fast` (pre-commit) and
`--full` are the gate.

**Approach.** Evolve 0.2.2 into **0.3 = the ideal spec**; don't rewrite. Much already matches (domains, `?T`
with absorbing `+`, floor `/` `%`, `INT64_MIN` niche, range types, `@memo`, indentation, `and/or/not`, `=>`
arms). The gaps are specific and listable, so the first deliverable is that list.

## 1. Domains

Each domain has an owner lane, a spec chapter, and a measurable bar.

| # | domain | what "done" means | bar |
|---|---|---|---|
| D1 | **Spec and design** | ideal spec moved into cp-lang `spec/`, one source of truth; every core area decided or deferred; two-implementer conformance pass finds no ambiguity | conformance disagreements -> 0 |
| D2 | **Core language** (front end + checker) | 0.3 syntax and typing: fresh domains with slots and escape rules, existential domain returns, range inference A, optionals, traits checked at definition, operators as trait calls, decorators, modules | every spec example checks; known-bad programs give the specified error |
| D3 | **Codegen and performance** | big-stack thread from the judge profile; 8-byte `?Int`; arrays off-stack per 18.3; per-declaration slots; zero-cost domains/iterators | runtime p50 <= 1.1x, max <= 1.5x; memory p50 <= 1.0x; `sizeof` checks |
| D4 | **Standard library** | full design doc (every module, signatures, naming), then implementation written only with user-available powers | 0 privileged items; each module tested and documented |
| D5 | **Diagnostics** | every error names the place, the rule, and the fix; library-authored messages | error corpus golden tests; cold readers fix each error in one try |
| D6 | **Interpreter, debugger, trace** | interpreter matches C++ on every program; HTML trace; debugger | 3-way differential fuzz clean |
| D7 | **Tooling** | fmt, lint, stress with shrinking, test, profiler, coverage, bench, doc, `cpl explain` | each tool on the corpus without crash; fmt idempotent |
| D8 | **Judge bundling** | one C++17 file, deterministic, <= 64 KB, judge profiles | every port bundles under the limit |
| D9 | **Editor/LSP** | VS Code extension, hover, go-to, completion, diagnostics, formatting on 0.3 | latency budget tests |
| D10 | **Ports and validation** | 100-500 hard corpus problems re-ported in 0.3 (rethought, not translated), differential vs accepted C++ | each port: differential pass, perf bar, size bar, friction logged |
| D11 | **Docs and learning** | reference, stdlib docs (generated), cheat sheet, course, playground on 0.3 | every snippet runs |

## 2. Workflows

**Roles.**
- **Me (architect/editor).** Own the spec, `decisions.md` and the friction queue; triage; write lane briefs; review
  every batch; only I change the spec. Bring Ryan only core-level decisions, one at a time, with options.
- **Lane agents.** One per domain lane, partitioned by crate so two agents never edit the same
  crate. Each gets a minimal brief (goal, spec chapters, tests to pass, files owned) to avoid context poisoning.
  They push to `main` behind `scripts/check.sh --fast`, rebasing first (repo convention).
- **Port agents.** Waves of 4-5 sessions x 5-10 corpus problems. Each port: rethink the solution, write 0.3 code,
  differential-test vs the accepted C++, measure tokens/runtime/memory/bytes, and file friction in a fixed template.
- **Cold readers / grillers.** Fresh sessions with only GOALS + the artifact: grill exemplars, stdlib docs and
  error messages; report hesitation and surprise.
- **Conformance pair.** Two cold sessions answer the same ~80 edge-case questions from the spec alone; every
  disagreement is a spec bug.
- **Reviewer.** After each batch, a review session reads the diff since the last review and files bugs,
  simplifications and spec drift (numbered report).

**The iteration cycle (one round = one batch).**
1. Port wave -> friction reports.
2. Triage: cluster friction into general mechanisms; reject one-problem helpers.
3. Design: core-level -> CORE-ALIGN entry and Ryan; library-level -> I decide and log in `decisions.md`.
4. Spec edit, then a compiler/stdlib batch with tests from the triggering ports.
5. Regression: all ports re-checked, differential + perf + size; the friction lint counts go down.
6. Grill and conformance on what changed.

**Order of work.**
1. **Now:** delta audit (0.2.2 vs ideal spec, per feature: same / rename / semantic change / new / delete);
   full stdlib design doc; set up lanes.
2. Ship mechanical syntax changes first (renames, deletions such as `@depth`-style counters, `print(Bool)`
   -> `yes(b)`), with a `cpl migrate` rewriter so the 370 existing ports move with the language.
3. Core semantics next (fresh domains + slots, escape rules, range inference, traits/operators), then codegen
   (big stack, niche `?Int`), then the stdlib redesign.
4. Port waves run continuously from step 2 on.
5. Deferred areas (program shape, `#set`, test roles) keep 0.2.2 behaviour until Ryan decides them.

## 3. Principles

1. **GOALS.md is the constitution;** coherence is a constraint on every change.
2. **Small core.** If the stdlib needs a power, either a user library gets it too or it goes into core
   explicitly, with a CORE-ALIGN entry. No privileged stdlib.
3. **Coherent stdlib.** One naming scheme (`len`, `?`-returning twins of trapping ops, `by:` keys, `first/last`),
   one way to do each thing, guessable from neighbours; every public item has a signature in `spec/40`, a doc,
   a test and an example.
4. **General before special.** A construct with one use means the general mechanism is missing.
5. **No secret choices.** Implicit representation choices only when never worse, and `cpl explain` shows them.
6. **Honest measurement.** Line/token wins exclude stdlib code; perf compares matched algorithms; claims cite a run.
7. **Main stays green.** Every change lands with tests; ports are regression tests; no batch without differential runs.
8. **Ryan's time is the scarcest resource.** Decide library-level things myself and log them; ask him only
   core-level questions, one at a time, with a recommended pick.
