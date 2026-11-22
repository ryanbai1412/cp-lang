// Codeforces 2207 B — Flashlight Window Simulation
//
// At each second t = 1..=l, increment one active slot, and whenever `t` hits a
// flashlight time `a[idx]`, the largest slot is removed (dropped from the
// front of the non-increasing window) and only `min(m, remaining_flashes + 1)`
// slots can ever matter — at most `n + 1` initially since only `n` flashes
// can remove slots before the final maximum is read.
//
// Incrementing the earliest minimum slot preserves non-increasing order and
// keeps the active window as balanced as possible, maximizing `vals[0]` after
// all flashes. Runs in O(l * min(m, n)) time using a `List<Int>` whose length
// is the active window size.

@!multitest
let (n, m, l): (1..=200_000, 1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=l = read()

var vals = [0 for _ in 0..min(m, n + 1)]
var idx = 0

for t in 1..=l:
    let best = (0..vals.len).min(by: i => vals[i])!
    vals[best] += 1
    if idx < n and t == a[idx]:
        idx += 1
        vals.remove(0)
        if vals.len < n - idx + 1: vals.push(0)

print(vals[0])
