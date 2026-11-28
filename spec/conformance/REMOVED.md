# Removed cases

A conformance case or new-pipeline test leaves only with a line here naming it and why
(`scripts/debt-ratchet.py`, `next_cases_removed`).

- `crates/cpl-next/tests/programs/a_own_19_generic_return_moves.cp`: renamed to
  `a_own_19_generic_return_lends.cp`. It expected `id(b)` to move `b`, but R05.5.3 says a
  generic fn's result that holds its argument lends it; today's `cpl` agrees and prints `2 1`.
- `spec/conformance/core-05/walk-body-cannot-touch-mut-source.cp` and
  `spec/conformance/core-12/intrinsic-wrong-signature.cp`: kept, but each printed a `Bool`, which
  R10.3.3 makes an error (E0606) before the error the case is about; they print `yes(b)` now.
- `spec/conformance/diagnostics/definition-cycle.cp`: kept, but it expected E0209, which no spec
  rule states: only the frozen `cpl`'s lowering sized `S<V>` while it typed `f`'s signature. A
  domain's count is read lazily, so nothing cycles; the case runs on `cpl-next` and prints `1`.
- `spec/conformance/lints/turbofish-call.cp`: L0204 is deleted (decision 664), since `::<` is now
  E0503 in the parser with the same fix. The case moved to `core-01/turbofish-is-rejected.cp`.
- `spec/conformance/std-seq/drop-negative-traps.cp`, `spec/conformance/std-seq/min-k-negative-traps.cp`,
  `spec/conformance/std-seq/step-of-zero-traps.cp` and `spec/conformance/std-seq/windows-zero-traps.cp`:
  renamed to `drop-negative-is-rejected.cp`, `min-k-negative-is-rejected.cp`,
  `step-of-zero-is-rejected.cp` and `windows-zero-is-rejected.cp` (0bb387920, the coordinator's
  ruling). Each passes a constant out of its parameter's range (`k: Len`), which R04.4.24 checks
  when compiled as E0902; a computed value out of range still traps at run time.
- `spec/conformance/core-07/rev-iter-is-walked-not-called.cp`: removed, its E0203 encoded the
  frozen `cpl`'s `foreign_iter_call` name list (an `.iter()`/`.into_iter()`/`.chars()` call on a
  concrete std type rejected by name), which no spec rule states. R08.1.4 finds a trait's method
  on a type that implements it, and R11.1.4 / decision 358 give trait methods the trait's
  visibility, so `a.rev.iter()` calls `Rev`'s `impl Seq` walk (coordinator ruling, option A).
  Replaced by `core-07/rev-iter-resolves-through-seq.cp` (`expect: ok`, `4 1 3` on all three
  runners).
- `spec/conformance/core-08/concrete-extension-beats-std-generic.cp`: its verdict flips, so it
  became `core-08/extension-hidden-by-trait-default.cp` (E0701, `engines: next`). Folds are
  default methods of their trait (decision 486), so `Seq.sum` is a member of `1..4`, and a member
  beats any extension fn (R08.1.4, decision 677): the concrete `fn sum(self: Range<Int>)` is
  hidden, with help to call it as `sum(1..4)`. Old `cpl` still accepts it and prints `7`.
