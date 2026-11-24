# noi02p3

Score: 3/5
Dream: 3/5

Port 3/5, dream 3/5 (`dream.cp`). Tree knapsack over who eats each fruit and how many the big head eats below it; `clash` (line 16) states the charging rule once and costs are `?Int`.

## Checklist failures

- idiomatic-code/R3: lines 26-30 hand-roll the min-plus convolution of the child tables as four nested loops with `min=`. Reported: `?Int` satisfies no `Add` bound, so `convolve(over: Semiring(min, +))` rejects it (E0606 quoted in the friction).
- idiomatic-code/R2: `eat(v, from: ?F)` with `if Some(a.to) == from: continue` (lines 19-22) walks the unrooted graph because `Rooted` drops arc fields. Reported.

## Owed rewrites

- `Head.Big` (lines 16, 20, 24, 33) where the dream writes `Big`; use the short form if the enum's variants are in scope.

## Language gaps

Both are in the friction.

## Dream notes

`let never = 1 << 40` is a sentinel infinity (idiomatic-code/R5), and `edge` nests a `map` of a `map(...).min()!` inside a comprehension. The dream should keep the port's `List<?Int>` with `None` as unreachable and `convolve(best[me], edge, over: Semiring(min, +))`, which is exactly the friction's ideal line.
