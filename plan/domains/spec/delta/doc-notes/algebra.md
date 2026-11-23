# algebra doc notes

- Contradiction: `spec/lib/ops-ranges.md` section 2 says "`MinIdx`/`MaxIdx` go" (replaced by `min(by:)`), but the stub still defines them. Documented the stubs; `join` follows the spec's tie rule for `min(by:)` (the left operand). (resolved: removed `MinIdx`/`MaxIdx`; `min(by:)` replaces them, decision 166)
- Contradiction: `spec/lib/ops-ranges.md` section 1 names the `MaxSub` identity `MaxSub.zero` (via `Add`), while the stub has `MaxSub.empty()`. Documented `empty()` as written. (resolved: kept stub `MaxSub.empty()`; ops-ranges §1 fixed)
- `fold`: an empty `xs` gives the identity of `m` (the spec says "folded left to right" only); an `m` without an identity is a compile-time error because the return type is `T`, not `?T` (the `Op` trait doc says folds over an identity-free `Op` return `None`; that applies to structures' `try_fold`, not this signature).
- `MaxSub.scale(0)`: chose that a non-empty summary scaled by 0 has best 0 (every element becomes 0) and the empty summary stays empty.
- `MinCount.monoid`, `MaxCount.monoid`: identity value is the far end of `T` with count 0, matching `Monoid.of(min)` / `Monoid.of(max)`; the spec does not state it.
- `Group.of`: chose `+` (negation) and `^` (self-inverse) as the only bare operators with a known inverse, as the stub's examples list.
- `point_action`: the spec does not say when it is correct; documented it as right for idempotent folds, where `n` leaves changed by one tag fold to one changed value.
- `desc`: "shows as `x` does" chosen for `impl Show for Desc<T>`, whose behaviour the stub leaves open.
