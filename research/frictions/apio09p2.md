# apio09p2 friction

## The suffix argmin needs `suffix_fold` and a tuple key, not `.rev.scan(min(by:))`
- **Where:** `spec.cp`: `let soonest = suffix_fold(min, by_start.map(i => Some((iv[i].r, i))))`.
- **What felt wrong:** the spelling the spec teaches, `by_start.rev.scan(min(by: i => iv[i].r))`,
  fails twice: `min(by:)` is not an operation value yet (E0303), and `.rev.scan` is indexed by
  the reversed cuts, not "the same cuts" (R07.1.6; `core-07/prefix-scan-cuts.cp` fails). The
  workaround carries the key and the id as an optional tuple.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `let soonest = by_start.rev.scan(min(by: i => iv[i].r))`.
- **General fix:** already routed: `min(by:)` as an op value (stdlib worklog, compiler worklog
  2026-11-22) and `Prefix` cut indexing (stdlib worklog, R07.1.6 request).
- **Severity:** medium.

## `Lift.of` needs its type arguments spelled out
- **Where:** `let chain = Lift<I, ()>.of(i => first_from(iv[i].r))`.
- **What felt wrong:** `Lift.of(...)`, as its own doc writes it, is E0302 (`V` resolves to
  `Int`); already in `std/GAPS.md` (tree: `Rooted` and `Lift`).
- **Goal it hurts:** express.
- **Ideal code:** `let chain = Lift.of(i => first_from(iv[i].r))`.
- **General fix:** infer `V` from the closure's parameter use (routed in GAPS.md).
- **Severity:** low.

## `Lift<V, ()>` does not compile to C++
- **Where:** the same `Lift<I, ()>.of(...)` line.
- **What felt wrong:** `cpl build` emits a `[](void, void) -> void` lambda for the unit op, so
  the solution runs in `cpl run` but cannot be submitted.
- **Goal it hurts:** submission.
- **General fix:** lower `()` to an empty struct in codegen (compiler worklog 2026-11-23).
- **Severity:** blocker for submission.

## Measurements
- Interpreter vs AC C++ (`corpus/submissions/5536701.cpp`, g++ -O2): 10/10 tests, 300/300
  `gen.py` cases at n <= 12 and 60/60 at n <= 2000 agree.
- n = 2e5 (ends up to s + 2e4): AC C++ 0.23 s; `cpl run` 12 min 2 s, same output; no native
  build (unit codegen gap above).
