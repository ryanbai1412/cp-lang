# Friction report: coci21c2p5

## Array `maximal` fails on a domain-keyed array
- **Where:** `suspects.maximal(held, ok: h => h.state.all <= 1, anchor: Anchor.Start)` with `suspects: [S]S` (old `spec.cp`).
- **What felt wrong:** E0301 "indexed by `S`, but the index is a `Int`" inside std/seq.cp:3173: the body indexes `self[r]` with an `Int` ordinal. On a `List` it is E0302 "needs a domain for `D`", so no array can call it.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `let group = iv.keys.maximal(cover, ok: t => t.state.all <= 1, anchor: Anchor.Start)`.
- **General fix:** `maximal`/`minimal` index by `D.nth(i)` (stdlib worklog). `spec.cp` writes the two-pointer loop by hand meanwhile.
- **Severity:** high.

## `track` closures don't infer their element type
- **Where:** `track(SegTree(..), add: (mut t, s) => t.apply(heights(s), 1), ..)`.
- **What felt wrong:** `s` defaults to `Int`, so `heights(s)` is E0302 until written `s: S`.
- **Goal it hurts:** express.
- **Ideal code:** unannotated closures, typed from the `maximal` call that consumes the `Tracked`.
- **General fix:** infer a closure parameter from its later use in the same statement chain (compiler).
- **Severity:** low.

## `Lift<V, ()>` C++ doesn't compile
- **Where:** `Lift<S, ()>.of(s => after[s])`.
- **What felt wrong:** `cpl run` answers, but the C++ has `std::vector<void>` and `[](void, void)`; std's own doc example fails the same way (perf worklog).
- **Goal it hurts:** submission.
- **Ideal code:** as written.
- **General fix:** codegen lowers `()` values and fields to an empty struct, everywhere `void` can't stand.
- **Severity:** blocker for submission.

## Measurements
- Max input (n = q = 2e5): AC C++ 0.19 s; interpreter 15.4 s; spec C++ does not compile (above). Emitted C++ 28,539 bytes.
