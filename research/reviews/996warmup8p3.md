# 996warmup8p3

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp` (the same code bar the `// Bounds:` line), without
the C++. Hits by time t are a count of per-period offsets, `first_true` finds the moment one wall
is done, and the other wall's remaining hits come from the balls heading for it, nearest first.
That is the editorial, but the closing arithmetic is not derived.

## Checklist failures
- idiomatic-code/R1: the search bound `0..=2e9 * L / n + 2` (line 41) and the closing
  `t1 + 2 * L - (if need < movers.len: movers[need - 1] else: 0)` (line 52) are not derived by
  the header: why `2e9 / n` periods suffice, and why `need == movers.len` falls to the `0` arm
  rather than reading `movers[need - 1]`.
- idiomatic-code/R4: `hits(t)` is computed twice per probe in the `first_true` predicate
  (`hits(t).left >= kl or hits(t).right >= kr`, line 41), each an O(n) count.
- idiomatic-code/R5: `dir: [Ball]0..=1` and `if dir[b] == 1` (lines 17, 21, 35) keep the
  direction as a number; an `enum Dir: Left | Right` read (or `Bool`) would name it.

## Owed rewrites
- `let h = hits(t); h.left >= kl or h.right >= kr` inside the predicate (a block lambda), so the
  count runs once per probe.
- Derive the `2e9 * L / n` bound and the `else: 0` case in `// Solution:`.

## Language gaps
- A `let` clause in comprehensions, for `for s in [place(b, t1 % (2 * L))]` (line 50): in the
  friction, with a run.

## Dream notes
The dream is the port, so it keeps the unexplained bound and closing formula (caps at 3).
It should derive both in its header and name the direction as an enum.
