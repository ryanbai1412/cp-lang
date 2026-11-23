// Source: Codeforces 2101 D, Mani and Segments, https://codeforces.com/contest/2101/problem/D
// Tuned variant of port.cp (same algorithm): both scans read `a` in place, the
// stacks hold sentinels so the pops test no option, values and positions are
// ranged to 32 bits, and the suffix minimum runs in place.

@!multitest
let n: 1..=200_000 = read()
domain Pos = 0..n
let a: [Pos]1..=n = read()

// For each start (in scan order), the first end that breaks it.
fn first_bad(rev: Bool) -> List<0..=200_000>:
    var cur: List<0..=200_000> = [n for _ in Pos]
    var lo: List<(val: 0..=200_001, pos: -1..200_000)> = [(0, -1)]
    var hi: List<(val: 0..=200_001, pos: -1..200_000)> = [(n + 1, -1)]
    lo.reserve(n + 1)
    hi.reserve(n + 1)
    for i in 0..n:
        let v = a[Pos(if rev: n - 1 - i else: i)]
        while v < lo[lo.len - 1].val: lo.pop()
        while v > hi[hi.len - 1].val: hi.pop()
        let b = min(lo[lo.len - 1].pos, hi[hi.len - 1].pos)
        if b >= 0: cur[b] min= i
        lo.push((v, i))
        hi.push((v, i))
    for i in (1..n).rev: cur[i - 1] min= cur[i]
    cur

let bad_r = first_bad(false)
let bad_l_rev = first_bad(true)
var total = 0
for i in 0..n:
    total += (bad_r[i] - i) * (i - (n - 1 - bad_l_rev[n - 1 - i]))
for k in 0..n - 1:
    total -= (bad_r[k] - (k + 1)) * (k - (n - 1 - bad_l_rev[n - 2 - k]))
print(total)
