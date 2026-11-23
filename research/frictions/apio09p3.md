# Friction report: apio09p3

## A let-bound `.max()` is `T`, not `?T`
- **Where:** `var tail = dag[c].filter_map(a => best(a.to)).max()` then `tail ?? 0` (old `spec.cp`).
- **What felt wrong:** std and R07.9.3 say `.max()` is `?T`; the compiler types it `T` (trapping) unless `??` or `!` follows directly, so E0309.
- **Goal it hurts:** hard-to-get-wrong, coherence.
- **Ideal code:** `let tail = xs.max()` then `tail ?? d`.
- **General fix:** lower bare `min`/`max` to the optional reduce on every `Seq` (filed in the compiler worklog from cf2229e and the idioms error triage). `spec.cp` now writes `.max() ?? stop`.
- **Severity:** high.

## `@memo(array)` recursion through a closure runs out of interpreter stack
- **Where:** `@memo(array) fn best(c: C) -> ?Int` recursing inside `dag[c].filter_map(a => best(a.to))`, on a 5e5-component chain.
- **What felt wrong:** R0601 in `cpl run`; the C++ answers. spec/lib/search-dp.md says `@memo` adds nothing per frame.
- **Goal it hurts:** evidence (interpreter == C++), submission.
- **Ideal code:** the `@memo` recursion as written.
- **General fix:** the memo fill wrapper adds no interpreter frames (compiler worklog, 2026-11-23). `spec.cp` fills `best` over `C.rev` instead, the editorial's reverse topological order.
- **Severity:** medium.

## Measurements
- Source bytes: spec.cp 1011, AC C++ 4675. Emitted C++ 14942 bytes.
- 5e5-vertex chain (max N, M): spec C++ 0.15 s, AC C++ 0.15 s; interpreter 1.0 s.
