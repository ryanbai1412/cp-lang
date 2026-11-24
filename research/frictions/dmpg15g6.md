# Friction report: dmpg15g6

The v0.2 port was a rollback Mo with `-1` sentinels for "no flavour", a hand-kept top-two pair and a manual undo log. The v0.3 port drops Mo: a `SegTree` of three-counter Misra-Gries summaries (`Monoid([], join)`) gives at most three candidates per query, and `flavour.buckets()` gives each flavour's sorted pieces, counted with `first_true`.

Gap: no std "how many of these sorted keys fall in l..=r" on a `List`, so `below`/`above` spell the two binary searches with `(0..xs.len).first_true(...)`.

## Dream

```cp
let counts = [at[h.flavour].count_in(l..=r) for h in heavy.fold(l..=r)]
```

## Measurements

gen.py --max (N = Q = 2e5, three flavours, long ranges): port 0.67 s, accepted C++ 0.36 s.
