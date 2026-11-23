# etsr23r3p3 friction

spec.cp failed to check: it copied two doc spellings that don't hold today, and
its `m` bound was narrower than the statement (`n = 1` has `m = 0`). Fixed in
spec.cp; the doc gaps are routed.

## `kruskal_tree` doc shows `w: e => desc(e.alt)`

- **Goal it hurts:** express and revise (writable from the docs)
- **Severity:** medium
- **Area:** stdlib
- **Status:** open (routed to stdlib)
- **Repro:** `kruskal_tree(roads, w: r => desc(r.alt))` is E0203 "an `E` has no
  member `alt`": `w` takes the edge key `E` (std/tree.cp `w: E -> W`), so the
  working form is `w: e => desc(roads[e].alt)`. The doc comment above
  `kruskal_tree` says `w: e => desc(e.alt)`.
- **General fix:** make the doc's prose spelling a runnable `example:` so
  `cpl-conform docs` catches it; or let `w` take the row (as `by:` would).

## Forced-online decode has no `ring` form

- **Goal it hurts:** express and revise
- **Severity:** low
- **Area:** stdlib / compiler
- **Status:** already reported (stdlib worklog H-RING-RECEIVER)
- **Repro:** `V.ring.step(v0, k * last)` is E0501 "can't handle `.ring` of a
  domain as a value yet", and `(0..=s).ring` is E0302 (a `Range` is not an
  array), though the `Ring.step` doc names `V.ring.step(v, last)` as the
  online decode. spec.cp writes `V.nth((v0.ord + k * last) % n)` and
  `(p0 + k * last) % (s + 1)`.
- **General fix:** `.ring` on any domain or range (positions as a cycle).
