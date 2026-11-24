# Friction report: yac7p5

There is no dream.cp; the ideal is below. The port splits the eggs by recursive halving. `place(lo, hi, eggs)` drops all but a spare at `mid - 1`, partitions the eggs by whether they broke, and recurses nearer side first to keep the elevator moving one way. `standing` records the elevator's floor.

## Dream

```cp
domain Floor = 1..=n
var floor_of: [Egg]?Floor = None
fn place(floors: Range<Floor>, eggs: List<Egg>): ...
print("!", floor_of.map(f => f!))
```

## Floors are plain `Int`s with a `0` placeholder
- **Where:** `var floor_of: [Egg]Int = 0`, with `place(lo: Int, hi: Int, ...)` over half-open integer ranges.
- **What felt wrong:** every egg's floor is set exactly once, so the `0` is a "not yet" value that the type does not tell from a floor. `lo..hi` ranges of floors are bare integers.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** the dream's `Floor` domain, a `[Egg]?Floor` and ranges of `Floor`.
- **General fix:** none; this is port spelling. Owner: idioms.
- **Severity:** low.

## All but the last element is an index slice
- **Where:** `eggs[..eggs.len - 1].partition(...)` next to `let spare = eggs.last!`.
- **What felt wrong:** "split off the last element" is spelled twice.
- **Goal it hurts:** express.
- **Ideal code:** `let (rest, spare) = eggs.split_last()!`.
- **General fix:** a `split_last() -> ?(List<T>, T)` (and `split_first`) on lists. Owner: stdlib.
- **Severity:** low.

The `--max` output differs from the accepted C++ because the transcript of an interactive run is not a unique answer. The interactor accepts both.

## Measurements

- **Tokens:** port 253, accepted C++ 379 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.07 s / 3 MB.
- **Emitted C++:** 17704 bytes.
