# Friction report: ioi99p6

## Dream

```cp
let h: [Row, Col]-30_000..=30_000 = read()
let best = max_by(area, for left in Col for right in left.take(100):
    let span = [Row] r => h[r, left..=right].minmax()   // kept incrementally
    two_pointers(Row, fits: rows => rows.map(span).spread() <= k))
print(best.corner)
```

## Monotone queues are `Window(max)` / `Window(min)`

The v0.2 port kept two hand-rolled monotone deques of row numbers. The rewrite pushes and
pops row maxima and minima on two std `Window`s. `Window.pop` needs the value being popped,
so the loop pops `hi[top]` and `lo[top]` again. A `Window` keyed by position (`pop_front()`)
would drop the repeat.

## Two pointers by hand

The longest run from each top row that stays within the spread is a `while let r = Row.at(bottom)` loop
with an `Int` `bottom` cut. A std two-pointer over a `Track` of the two windows exists
(`std.window`), but it reports per-left maxima as a function, not the bottom row each top
needs here, so the loop stays.

## A best-so-far with ties

`best` is an optional `(area, corner)` that keeps the first strict maximum, matching the
accepted C++. A `max_by` that keeps the first maximum would say that directly.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (new `gen.py`). On the max case
(700 x 700, C = 10) the outputs are identical; the port runs in 0.22 s and the C++ in 0.70 s.
Corpus tests pass.
