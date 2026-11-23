# noi10p2 friction

The spec wants `split_best(Note.cuts, range: s => s.shift(len), best:).take(k).sum()`; today it
walks `Int` cut counts and stops with a `for` loop after k values.

## `split_best` takes only `Range<Int>`

- **Where:** `best(from: Note.cuts, ends: Range<Note.cuts>)` is rejected: the body of
  `split_best` compares `r.l` with the position and adds `p + 1` (std/search.cp:306, :357), and
  a cut-range literal passed to a generic `Range<D>` turns into a `Range<Note>` piece.
- **Repro:** `domain V = 0..3; fn id<D>(r: Range<D>) -> Range<D> = r; let r: Range<V.cuts> = id(V.cuts(0)..V.cuts(2))`
  is E0302 "found `Range<V>`"; `split_best(V, range: c => c..V.last!, best: ...)` is "can't
  compare a `V.cuts` with a `V`".
- **Goal it hurt:** hard to get wrong (cuts and ends become plain `Int`s) and express.
- **General fix:** `split_best` over any `Range<D>`; generic `Range<D>` keeps a cut literal's kind.
- **Routed:** stdlib worklog (`split_best`), compiler worklog (inference).

## `c.shift(r)` is not implemented

- **Where:** `s.shift(len)`, the ends of the chords from `s` (R04.3), is E0203.
- **Routed:** already in the compiler worklog (from cf2124f2).

## A generator with a nested fn is not lazy

- **Where:** `split_best` declares `better`/`push_entry`, so `.take(k)` and a `for` with `break`
  still run every split: O(n^2) `best` calls. Tests stay at sizes the interpreter drains in time
  (the largest, n = 1e5 with R - L = 1, takes 6.3 s).
- **Repro:** a `yields` fn with `fn bump(): i += 1` beside `yield i` runs to the end under
  `for x in nat(): break` (1000 iterations instead of 1), interpreted and in C++.
- **Goal it hurt:** performance (the obvious code is quadratic).
- **Routed:** compiler worklog.
