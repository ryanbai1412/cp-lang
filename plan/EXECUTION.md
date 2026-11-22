# Execution plan: the next 12 hours and after

Owner: Devin (parent session). Status: in progress, updated 2026-11-21 23:10 PST.
Companion to `plan/ARCHITECTURE.md` (how the code is built) and
`plan/DOMAINS.md` (each domain's north star).

## 1. The goal and what "ready" means

Ryan's target: by 8:00 PST tomorrow ("ideally 8AM actually"), a language and toolchain usable
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

**Approach:** evolve the existing compiler (see "Decision: evolve, don't
rewrite" in `ARCHITECTURE.md`). The stdlib is already CP Lang; the work is
merging `spec/std` and `stdlib` into one `std/` with v0.3 names and real
bodies, making v0.3 the only mode, and closing checker gaps in order of
corpus failures. The compiler is usable at every step.

Measured 2026-11-21: 97 of 100 `spec.cp` ports type-check against the
stubs; 5 check against the implemented stdlib; none run in v0.3 form.

Measured 23:05: with 954 ported bodies, 2 of 54 tested v0.3 ports pass. The
blocker is not mostly missing bodies: `cpl run` re-types every program in
`cpl-ir`'s lowering, which has no associated types (`G.Arc`), trait aliases,
record rows (`..R without (u, v)`) or the implicit conversions of decision
183, so it rejects the v0.3 stubs' signatures before any body runs.

**Consequence for 8:00.** The contest path tomorrow is today's default
`cpl` (no `CPL_SPEC`), which runs the language the site teaches; streams 8,
10, 11 and 12 harden it. v0.3 runs end to end through a new reference
interpreter in `cpl-core` (architecture step 5, pulled forward): it runs
the checked program dynamically, so the stubs' type features need no
monomorphization, and it becomes the engine the stdlib-body streams test
against. The fast path follows it after `cpl-ty` (step 4).

## 2. Workstreams

Each stream has an owner, a done-test, and its dependencies. "Me" means
this session; "child" means a child session (allowed since 23:00), reviewed
by me before its work counts.

| # | Stream | Owner | Done when | Depends on |
|---|---|---|---|---|
| 0 | Tech-debt cleanup | child (`proto/ports` move, toolchain), me (review, docs migration) | prototype gone, one test home per kind, `just check` under 1 min, merged branches deleted | none |
| 1 | Spec finalization | me | round-3 rebuild under 5 holes; 0 vocabulary hits; all blocks parse | 0 (paths) |
| 2 | Spec safeguards | me | checks below run in `just check` | none |
| 3 | Harness (`cpl-conform`) | me | one runner and scoreboard JSON for conformance, corpus, doc examples and differential | none |
| 4 | One `std/`: v0.3 signatures with bodies; one mode | me | the corpus row of the table above | 3 |
| 5 | Checker and IR to the target design | me | ch1–ch10 conformance; intrinsics table only in `call.rs` | 3 |
| 6 | Stdlib bodies | two children (sequences and ranges; graphs and math) | module doc examples run | 4 |
| 7 | C++ codegen and perf | me | budget rows above; `just bench` report | 4 |
| 8 | Diagnostics | child | every code in `spec/diagnostics.md` has a case and a fix | 1 |
| 9 | Reference interpreter (`cpl-core`), then traces | me | v0.3 corpus ports run on it; conformance runs on it | none |
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

| Time | Me | Children |
|---|---|---|
| 23:00–23:30 | plan update; children launched; tech-debt review | tech-debt (`proto/ports`, mise/just/uv/pnpm), judges, LSP, diagnostics, stress, stdlib x2, site |
| 23:30–04:00 | `cpl-core` reference interpreter: v0.3 corpus ports and conformance on it | as above; I review each child's pushes as they land |
| 04:00–06:00 | interpreter gaps by corpus failure count; spec holes found on the way; rebuild round 3 | contest dry-run child exercises today's `cpl` on fresh problems |
| 06:00–08:00 | fix every red contest-path row; regenerate scoreboard; morning report | children wrap up |

## 6. Delegation

Ryan allowed child sessions at 23:00 ("you're the owner and you're on the
line if things don't end up getting shipped"). Delegated: tech debt (0),
stdlib bodies (6), diagnostics (8), stress (10), judges (11), LSP (12),
site (13). Each has a fixed interface, a done-test and a time box; I review
every push and own the result.

## 7. Morning report

At 8:00 PST: the readiness table filled with measured numbers, what's red
and why, the decisions made overnight, and the next three things to do.
