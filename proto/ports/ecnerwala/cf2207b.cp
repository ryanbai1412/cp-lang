// Codeforces 2207 B — Flashlight Window Simulation
//
// At each second `t = 1..=l`, increment one active slot; whenever `t` hits a
// flashlight time `a[idx]`, the adversary removes the largest active slot
// (`vals[0]`). Only `min(m, remaining_flashes + 1)` slots can ever survive to
// the end — at most `n + 1` initially since `n` flashes remove `n` slots before
// the final maximum is read.
//
// Exchange argument for the earliest-minimum greedy: because every future flash
// deletes the current maximum, moving an increment from a larger slot to a
// strictly smaller slot never decreases the surviving value after any prefix
// of top-slot deletions. Thus incrementing the minimum slot is optimal, and
// breaking ties toward the earliest index (`vals.argmin()!`) preserves
// non-increasing order `vals[0] >= vals[1] >= ...`.
//
// The statement bounds `n, m, l <= 2000` (with `min(m, n) * l` small), so
// direct simulation over `vals: List<Int>` runs in O(l * min(m, n)) time.

@!multitest
let (n, m, l): (1..=2_000, 1..=2_000, 1..=2_000) = read()
let a: [0..n]1..=l = read()

var vals = [0 for _ in 0..min(m, n + 1)]
var idx = 0

for t in 1..=l:
    vals[vals.argmin()!] += 1
    if idx < n and t == a[idx]:
        idx += 1
        vals.remove(0)
        if vals.len < n - idx + 1: vals.push(0)

print(vals[0])
