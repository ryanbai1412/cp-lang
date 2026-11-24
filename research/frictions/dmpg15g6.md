# Friction report: dmpg15g6

The v0.2 port was a rollback Mo with `-1` sentinels for "no flavour", a hand-kept top-two pair and a manual undo log. The v0.3 port drops Mo: a `SegTree` of three-counter Misra-Gries summaries (`Monoid([], join)`) gives at most three candidates per query, and `flavour.buckets()` gives each flavour's sorted pieces, counted with `first_true`.

Gap: std `count_in` does not take a range of a domain, so `below`/`above` spell the two binary searches with `(0..xs.len).first_true(...)`. On a fresh `cpl`, `at.count_in(P(3)..=P(8))` on a `List<P>` gives `error[E0302]: count_in (std.seq) does not work on the types it was given here: can't compare a P with a P.cuts` (`std/seq.cp:2355`); `count_in(3..8, by: p => p.ord)` works. General fix: `count_in` over `Range<D>` for a domain `D` (owner: stdlib). The `Monoid([], join)` is not associative (a Misra-Gries merge is order dependent); it keeps every flavour over a quarter under any bracketing, but the type claims a law `join` lacks (owner: stdlib, a mergeable-summary fold).

## Dream

```cp
let counts = [at[h.flavour].count_in(l..=r) for h in heavy.fold(l..=r)]
```

## Measurements

gen.py --max (N = Q = 2e5, three flavours, long ranges): port 0.67 s, accepted C++ 0.36 s.
