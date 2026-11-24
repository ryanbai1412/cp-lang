# Friction report: ioi20p5

## Dream

```cp
@interactive fn count_mushrooms(n: 2..=20_000, use_machine: List<Mushroom> -> Int) -> Int
enum Slot: Known(Species) | Unknown(Int)
let trees = Cache((k, x, y) => decision_tree(k, x, y))
var known: [Species]List<Mushroom> = [A: [Mushroom(0)], B: []]
while next < n and next < 199: classify(shape(known))
for batch in Mushroom.after(next).chunks_by(known.larger.len): interleave(batch)
print(known[A].len + counted)
```

## The interactor lives in the solution

The task is interactive: `use_machine` is the grader's. The port reads the hidden species and
simulates it, so query limits are not enforced. An `@interactive` entry, or the package's
interactor driving `use_machine`, would let the port be the contestant's function as is.

## Species is `0..=1`, not an enum

`hidden`, `Known` and `known` use `0..=1` for A/B because the species doubles as a mask bit
and as the table index of `known`. An enum usable as a domain index and convertible to a bit
would name the two species.

## A memo keyed by shape

`trees` is a `Map<(Int, Int, Int), Tree>` filled by `if not in: insert` before each read. A
std memo (`Cache(f)` or `map.get_or(k, f)` returning the stored value) would say it once.

## `adjacent().count(..)` is slower than the index form

`answer` was `q.adjacent().count(p => ...)`; the max case took 1.5 s against 0.7 s for
`(1..q.len).count(i => ... q[i - 1] ... q[i] ...)`. The adjacent-pairs walk should cost
what the index walk does; reported to Core.

## Output size

The C++ is about 34 KB, over the generic judge's 32 KB target (W0201).

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (`orig.cpp`, the AC
function plus a stdin `main` simulating `use_machine`), the corpus tests pass, and the max
case (n = 20000) matches the C++ in 0.7 s.
