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
- `spec/conformance/std-search/bsearch-found-follows-the-element-type.cp`: renamed to
  `bsearch-found-follows-the-range-type.cp`. Decision 672 retires the `Search` trait, so its
  generic `edge<D: Search>(..) -> D.Found` has no meaning left: `bsearch` is two methods, a
  `Cut<D>` on `Range<D>` and `?F64` on `RealRange`, chosen by the receiver (R08.1.4). The new
  case compares the whole Int cuts, `(8..8).l` and `(4..4).l`, where the old one printed one
  side (8 and 3). It adds the domain `last: true` search. `real-bsearch-is-a-range-method.cp`
  covers the real search in its own right.
