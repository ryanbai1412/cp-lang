# Execution plan: the next 12 hours and after

Owner: Devin (parent session). Status: proposal, 2026-11-21 22:40 PST.
Companion to `plan/ARCHITECTURE.md` (how the code is built) and
`plan/DOMAINS.md` (each domain's north star).

## 1. The goal and what "ready" means

Ryan's target: by about 10:30 PST tomorrow, a language and toolchain usable
in a real contest with minimal bugs and friction.

Measured, "contest-ready" means:

| Check | Bar |
|---|---|
| Write, run, build, submit loop | `cpl run`, `cpl build`, `cpl test`, `cpl stress` and `cpl judge` work on any corpus problem |
| Corpus | 100 of 100 `spec.cp` ports run in both engines and match expected outputs |
| Conformance | at least 90% of the 503 cases met, and none of the core ch1–ch10 cases missed without a logged reason |
| Output | every corpus port under 64 KB; p95 under 32 KB; runtime p50 at most 1.0x originals |
| Editor | LSP gives diagnostics, hover with types and sizes, completion and go-to on the 100 ports |
| Bugs | no known wrong-answer bug in either engine; differential fuzz clean for 30 minutes |
| Docs | reference generated from the stdlib; error index complete |

**Honest assessment:** the full v0.3 rewrite in `ARCHITECTURE.md` won't fully
land in 12 hours. The plan runs two tracks so the morning result is usable
either way:

- **Track A, ship:** make today's pipeline accept the v0.3 spec
  syntax and semantics that the 100 `spec.cp` ports use, and harden it.
  This is what Ryan uses tomorrow.
- **Track B, build:** the new spine (syntax -> HIR -> types -> MIR ->
  interpreter -> C++) in new crates, behind `cpl --next`, taking over
  chapters as they pass their conformance cases.

Track A gets the first hours, because it decides what works in the morning.

## 2. Workstreams

Each stream has an owner, a done-test, and its dependencies. "Me" means
this session; "child" means a candidate for delegation once Ryan allows new
child sessions.

| # | Stream | Owner | Done when | Depends on |
|---|---|---|---|---|
| 0 | Tech-debt cleanup | existing child, then me | prototype gone, one test home per kind, `just check` under 1 min, merged branches deleted | none |
| 1 | Spec finalization | me | round-3 rebuild under 5 holes; 0 vocabulary hits; all blocks parse | 0 (paths) |
| 2 | Spec safeguards | me | checks below run in `just check` | none |
| 3 | Harness (`cpl-conform`) | me | one runner and scoreboard JSON for conformance, corpus, doc examples and differential | none |
| 4 | Track A: v0.3 on today's compiler | me | the corpus row of the table above | 3 |
| 5 | Track B: new spine | me | ch1–ch5 conformance on `--next`; stdlib prelude and seq in CP Lang | 3 |
| 6 | Stdlib in CP Lang | children, one module each | module doc examples run on `--next` | 5 (types and MIR) |
| 7 | C++ codegen and perf | me | budget rows above; `just bench` report | 4 |
| 8 | Diagnostics | child | every code in `spec/diagnostics.md` has a case and a fix | 1 |
| 9 | Interpreter, traces, debugger | me (with 5) | one engine; traps show source values | 5 |
| 10 | Stress and contest tools | child | `cpl stress` with brute, generator and shrink on 10 corpus problems | 4 |
| 11 | Judges and submission | child | profiles for Codeforces, AtCoder, DMOJ, CSES, USACO; size warning at 32 KB | 7 |
| 12 | Editor and LSP | child | LSP row above; VS Code extension installs from `editors/` | 4 |
| 13 | Docs, site, playground | site child (running) | its own `site-plan.md` phase 1 | 1, 6 |
| 14 | Corpus and scoreboard | me | `corpus/problems/*` all green; scoreboard regenerated | 3, 4 |

## 3. Spec: finish, then stop

1. Fix the 32 round-2 holes, root causes first:
   - container stubs (`List`, `Str`, `Map`, `Set`), `Graph: Next`, `abs`,
     popcount, subset membership;
   - one decision each for the 12 silent rules;
   - the 15 contradictions, mostly unfinished renames.
2. Update `spec/STATUS.md` and the remaining old paths (`proto/ports` in
   `corpus/gold/GOLD.txt`, `corpus/wishes` in `plan/DREAM-BRIEF.md`).
3. Run rebuild round 3. If it finds under 5 holes, the spec is v0.3.0. If
   not, fix and run one last round. After round 4 we stop reviewing
   rounds: remaining holes are fixed as implementation finds them.
4. From then on the implementation drives the spec: every ambiguity found
   while coding is a spec fix, a decision and a conformance case.

## 4. Spec safeguards (stream 2)

Mechanical checks in `just check`, so the drift classes we hit can't come
back:

| Check | Catches | Status |
|---|---|---|
| `spec-vocab` | banned spellings from `spec/GLOSSARY.md` | done |
| `spec-stub --check` | stub bodies, missing doc labels | done |
| `spec-blocks --ratchet` | spec examples that used to check and now don't | done (29 blocks) |
| block tags | every block is `ok`, `error E…`, `fragment` or `todo`; `ok` must check | next |
| symbol resolution | every method used in spec prose, blocks or conformance resolves to a stub | next |
| removed names | every name the spec calls removed is absent from stubs and examples | next |
| one home per fact | `spec/lib` may not restate a signature from `spec/std`; it links | next |
| stale references | every path and rule ID cited in `spec/`, `plan/` and code exists | next |
| rule coverage | every rule ID has a conformance case (`conformance.py --rules`) | exists; to gate |
| generated counts | STATUS numbers come from the scoreboard, not hand edits | next |
| decision numbers | assigned by script, so branches can't collide | next |

## 5. Timeline (PST)

| Time | Me | Others |
|---|---|---|
| 22:45–23:30 | tech-debt finish and verify; harness skeleton; spec hole batch 1 (stubs) | tech-debt child wraps up; site child |
| 23:30–02:00 | Track A: the v0.3 forms the corpus `spec.cp` ports need, by failure count; spec safeguards | children: diagnostics, judges, LSP (if allowed) |
| 02:00–03:00 | spec holes batch 2; launch rebuild round 3 | rebuild child (if allowed) |
| 03:00–07:00 | Track B spine: syntax, HIR, types, MIR, interpreter; first stdlib modules | stdlib children once types and MIR are in (if allowed) |
| 07:00–09:00 | codegen and perf pass; differential fuzz; corpus green | |
| 09:00–10:30 | hardening: fix every red row; regenerate docs and scoreboard; morning report | |

## 6. Delegation

Ryan asked for no new child sessions for now. Until that changes, I do
everything above myself and the timeline stretches. When allowed, I'd
delegate only streams with a fixed interface and a test that defines done:
diagnostics (8), stress tools (10), judges (11), LSP (12), stdlib modules
(6) and rebuild rounds. Each gets a 45–60 minute cap, a check-in every 15
minutes, and my review of the diff before it lands on `main`.

## 7. Morning report

At 10:30 PST: the readiness table filled with measured numbers, what's red
and why, the decisions made overnight, and the next three things to do.
