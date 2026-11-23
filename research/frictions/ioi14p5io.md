# ioi14p5io — Friend

Every person but the first joins through a host and a protocol, so the
input's join records are keyed by `P` minus its first position. spec.cp
reads them as a plain `[n - 1]` array and indexes it by `v.ord - 1`.

## An array over a run of domain positions can't be declared
- **Where:** spec.cp:
  ```cp
  let joins = read<[n - 1](host: P, how: Protocol)>()  // person v joins in joins[v.ord - 1]
  ...
      let (j, b) = (joins[v.ord - 1], best[v])
  ```
- **What felt wrong:** R04b.1.5 says `[r]T` for a range `r` of `K` is
  indexed by `K` and valid on `r`, which is exactly "one join per person
  after 0"; instead the port re-derives the key with `v.ord - 1`, an
  off-by-one the domain was meant to remove. All three spellings fail on
  `domain P = 0..4`:
  `let a = read<[P.first!.after]Int>()` parses as comparisons (`a` is a
  `Bool`, E0305 on `a[P(1)]`); `read<[P.drop(1)]Int>()` is E0504 "a fresh
  type made from values"; `let a = [P.first!.after] v => v.ord` checks but
  `a[P(1)]` is E0301 "indexed by r, but `P(1)` is a P".
- **Goal it hurts:** hard-to-get-wrong, express.
- **Ideal code:** `let joins = read<[P.first!.after](host: P, how: Protocol)>()`
  and `joins[v]`.
- **General fix:** extend the landed `[r]T` for `Range<Int>` (e00a55322) to
  `Range<D>` of a declared domain, keyed by `D` (not a fresh domain), in
  type position too, so `read<[r]T>()` parses. Helps every parent-array
  read (`btoi17p6`'s `up[v.ord - 1]`). Routed to the compiler lane.
- **Severity:** medium (the `ord - 1` form works).

Measurements: spec.cp 282 contestant tokens vs 282 for the AC C++'s
`solve` + `findSample` (its grader's I/O not counted). On the n = 1e5 test
(tests/11): `cpl run` 0.33 s / 65 MB, the emitted C++ (12,434 B) 0.01 s /
6.6 MB, the AC C++ 0.01 s / 6.4 MB.
