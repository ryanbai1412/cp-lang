# compiler

Segment: B (metric). Owns: `crates/cpl-syntax`, `crates/cpl-check`, `crates/cpl-ir`, `crates/cpl-interp`, `crates/cpl-prover` (proofs, range analysis), and the shared-core interfaces ([ARCHITECTURE.md](ARCHITECTURE.md#interfaces-other-domains-use)). Status 2026-11-22: review packet; three branches queued to land. Detail: [PLAN.md](PLAN.md). Numbers: [metrics.toml](metrics.toml). Log: [worklog.md](worklog.md).

## Mission

Every program the spec defines means exactly what the spec says, in the interpreter and in the C++, and every program it rejects is rejected with the rule it breaks. One front end, one IR, one interpreter engine, no stdlib names in the compiler.

## Covers

- Lexer, parser, syntax tree (`cpl-syntax`); names, types, domains, lends, the incremental database and its query API (`cpl-check`); lowering to the typed IR, monomorphization (`cpl-ir`); the checked interpreter, its VM, traces and replay (`cpl-interp`); the integer prover (`prover`).
- The shared core: the five interfaces other domains build on, their invariants and their contract tests.
- Making failing conformance cases pass, and interpreter == C++ on every case.

Not covered (owner instead):

- C++ emission, layout, C++ size and speed: **perf** (`cpl-codegen`). The interp-vs-C++ number is shared: a disagreement is fixed by whichever side is wrong.
- Diagnostic wording, codes, the error index: **diagnostics** (`cpl-diag`). Compiler decides *when* an error fires; diagnostics decides *how it reads*.
- What the language means and the cases themselves: **spec** (`spec/core`, `spec/conformance`). Compiler never edits a case; disputes go to the spec worklog.
- Library bodies: **stdlib** (`std`). Trace viewer, stress: **debugger**. LSP/CLI: **tools**.

## Success criteria for v0.3

1. `cpl-conform conformance` passes every case (today 1617/1711).
2. `cpl-conform diff`: interpreter == C++ on every case with expected output, zero skipped (today 1242/1336, 94 skipped).
3. One interpreter engine: the VM runs every IR construct, the tree-walker fallback is deleted, and the playground (wasm) runs the same VM (today every one of the 13 corpus programs that run falls back at least once; the VM is compiled out on wasm, `crates/cpl-interp/src/lib.rs:16`).
4. No stdlib privilege in `cpl-check`/`cpl-ir`/`cpl-interp`: no std type name as a string literal outside one listed intrinsics table (today 146 occurrences of 35 names).
5. No edition branches (today 120 `is_v03()`/`spec_edition()` call sites), no owned non-test file over 2000 lines.
6. Every fix lands with the case that exposed it; front end stays under 500 ms on every corpus program.

## Milestones

- **M1 Queue landed, known bugs fixed.** Exit: `wip-split-vm`, `wip-conformance-queue`, `wip-product-domain` on main or deleted as superseded; `std-seq/slice-max-optional`, `diagnostics/max-of-a-slice-is-optional`, `std-compress/wrapper-fn-fresh-domain`, `core-04/rebase-fresh-domain` and every `core-04/prod-*` and `core-04/f*` product case pass; conformance >= 1660/1711.
- **M2 Spec-complete.** Exit: conformance 1711/1711; diff 0 skipped and 0 failing; `wrong output` and `E0302` causes in `--causes` both 0.
- **M3 One engine, no privilege.** Exit: `CPL_VM_STATS` fallback count 0 over corpus and conformance, tree-walker code deleted, cpl-wasm on the VM; std-name literals 0 outside the intrinsics table; edition call sites 0; owned files over 2000 lines 0 (tests excepted).

## Metrics

Every value measured 2026-11-22 on main `4b8f504f`; commands in [metrics.toml](metrics.toml).

| Metric | Command (short) | Today | Target | M |
|---|---|---|---|---|
| Conformance | `cpl-conform conformance` | 1617/1711 | 1711/1711 | M2 |
| Interp == C++ | `cpl-conform diff` | 1242/1336, 94 skip | 1336/1336, 0 skip | M2 |
| Distinct failure causes | `cpl-conform conformance --causes` | 47 | 0 | M2 |
| VM fallbacks (corpus) | `CPL_VM_STATS=json cpl run` each runnable `spec.cp` | 13/13 programs, 4522 executions | 0 | M3 |
| Std names in compiler | std type names as `"Name"` literals in check/ir/interp | 146 (35 names) | 0 outside table | M3 |
| Edition branches | `grep is_v03()/spec_edition()` | 120 | 0 | M3 |
| Owned files > 2000 lines | `debt-ratchet.json` allowlist, owned crates | 14 | 0 non-test | M3 |
| Owned `#[ignore]`s | `grep '#[ignore'` in owned crates | 9 | 0 | M3 |
| Owned-crate tests | `cargo nextest run --release -p cpl-{syntax,check,ir,interp,prover}` | 429 pass, 9 skip, 5 s | all pass | all |
| Front-end latency | `cpl check` largest corpus `spec.cp` (74 lines) | 0.02 s | < 0.5 s | all |
| Full conform run | `time cpl-conform all` | 23.3 s | < 30 s | all |

Watched, owned elsewhere: corpus 12/100 (46 untested; idioms/stdlib), docs examples 2012/2699 (stdlib), spec blocks 19/185 (spec). Compiler fixes any failure in these whose cause is in its crates.

## Judgment rubric

A cold agent scores 1-5 with a reason, on the last ~20 compiler commits:

1. **General, not special?** Does each fix work through a general mechanism, with no std name matched, no new edition branch, no one-case path?
2. **One place?** Does a language change touch one lowering (one IR construct, read by both back ends), or did it duplicate logic in check, IR, VM and tree-walker?
3. **Traceable?** From a rule ID (`R04.2.11`), can you find the code that implements it in under 2 minutes?
4. **Honest rejection?** Does the checker reject what the spec rejects, and say nothing it can't stand behind (no false E0302 on valid code)?
5. **Readable as a spec?** Could a reader learn the rule from the interpreter code that runs it?

Probes (cold agent, pass/fail + time + files touched):

- **Fix a case:** given one failing conformance case (e.g. `std-seq/slice-max-optional`), make it pass from `ARCHITECTURE.md` alone. Tests 1, 2, 4.
- **Add a construct:** add a small IR expression kind to both back ends. Tests 2; files touched should fall as M3 lands.
- **Find the rule:** five rule IDs, find their implementation. Tests 3, 5.

## Improve (highest impact first)

1. Land the queue (`wip-split-vm` first; others rebase on it).
2. Product domains end to end (`domain Cell = (Row, Col)`, `[Cell]T`, `p.step(d)`): 18 `wrong output` + 16 `E0302` failures cluster in `core-04`.
3. Seq results on slices are `?T` (`max`, `min`), and a fn that mints a domain mints a fresh one per call (`compress()` wrapper).
4. Generic inference holes: explicit type args on static calls (`SegTree<V, Int>.of`), domain params from a seq's keys, `Graph`/`Rooted` V inference, struct-level bounds in methods.
5. Ownership leftovers: E0608 move-in-loop/branch, lends on keyed slices.
6. Delete the tree-walker: VM lowering for every fallback tag, then cpl-wasm on the VM.
7. Std names out of `cpl-check` and `lower/call.rs` into one table (with stdlib and tech-debt, per the builtin-floor question); split `lower/call.rs` (13k) and `cpl-interp/src/lib.rs` (5.6k).
8. Wire `prover` into range facts: today only `cpl-lint` uses it; check removal lives in `cpl-ir/src/lower/range.rs` and codegen's `ranges.rs`.

## Decisions for Ryan

Filed in [plan/QUESTIONS.md](../../QUESTIONS.md) (1 and 2 by compiler, 3 by tech-debt), each with a default the domain works on meanwhile:

1. **How product positions lower:** a position of `domain P = (R, C)` is a tuple of factor positions in the IR (cheap neighbours, `(r, c)` destructures), with `nth` and array offsets computed row-major; not one packed row-major `Int`. Default: tuple (what `wip-product-domain` does).
2. **One VM, no tree-walker:** delete the tree-walker once the VM covers everything, and let the C++ diff be the only second opinion; not keep it as a reference oracle. Default: keep both until fallbacks reach 0, then delete.
3. **The builtin floor** (filed by tech-debt, "which names may the compiler know"): compiler backs option (a), a small floor declared in `std` with an `@intrinsic` marker any library may use. Today 146 literals of 35 std type names in check/ir/interp, beyond the 89 call names tech-debt counts in `lower/call.rs`. Default: the names only go down; first step moves them into one table.

## Ryan's role

Sign off the targets above once. Answer the architecture calls below; everything else is logged in `decisions.md` and reported as a number.
