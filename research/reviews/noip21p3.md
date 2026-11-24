# noip21p3

Score: 4/5
Dream: 4/5

Scored cold against the checklist and the port. The header states the key fact: the operation swaps the two neighbouring gaps, so the gaps can be ordered freely and the best order is a valley. `rest(i, s)` is that choice, one gap per step from small to large.

## Checklist failures

- The gaps are `[a[i + 1] - a[i] for i in 0..n - 1]` (friction: no `diffs()`).
- The memo key `s: 0..=top` with `top = n * (a[n - 1] - a[0])` is a correct but loose bound. Only sums reached from sorted gaps occur, and `@memo` hashes them. An array memo would need the tighter bound, which the comment explains.
- `-s * s` at the base and `n * (2 * d * s + lift * d)` fold the variance algebra into the recursion without naming the terms. The comment above `rest` covers them.

## Gaps that need the language

- None.
