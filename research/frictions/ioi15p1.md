# Friction report: ioi15p1

## Dream

```cp
let (n, k, l) = read<(1..=1e7, 1..=n, 1..=1e9)>()
domain Team = 0..n
let pos = read<[Team]0..l>()
var cw: [Team.cuts]Int = 0
for t in Team: cw[t.r] = 2 * pos[t] + cw[t.r.prev(k, clamp: true)]
var ccw: [Team.cuts]Int = 0
for t in Team.rev: ccw[t.l] = 2 * (l - pos[t]) + ccw[t.l.next(k, clamp: true)]
print(Team.cuts.map(c => cw[c] + min(ccw[c], l + ccw[c.next(k, clamp: true)])).min()!)
```

## A cut can't step by k

- **Where:** `cw[Team.cuts.at(Int(t.r) - k) ?? Team.cuts.first!]` and the two `next` twins.
- **What happened:** `t.r.prev(k)` is E0203 "a `T.cuts` has no method `prev`" in `cpl run`;
  positions have `v.prev(k)`, cuts only `.prev`. The port goes through `Int` and back with
  `Team.cuts.at`, and spells "stop at the end" as `?? first!`.
- **General fix:** `c.prev(k)` / `c.next(k)` on cuts like on positions, plus a saturating step
  (`clamp:`), the "at most k back" every prefix DP with a window writes.
- **Goal it hurts:** express, hard to get wrong. **Severity:** low.

## A dependent bound can't share a tuple read

- **Where:** `let n = read()`, `let k: 1..=n = read()`, `let l = read()` on three lines.
- **What happened:** `read<(1..=1e7, 1..=n, 1..=1e9)>()` names `n` before it exists.
- **General fix:** tuple reads whose later bounds name earlier fields.
- **Severity:** low.

## Measurements

Contestant-written tokens: port 291, original 222 (`corpus/audit.py` count). Emitted C++: 11183 bytes (`cpl build`, judge `generic`).
The accepted C++ is a grader function with no main, so the max input is timed only by `corpus/generators.py`.
