// Mock CCC '23 Contest 1 J3 — Distant Pair Sum Counting (CP Lang v2 spec)
// original: corpus/submissions/5333116.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Count pairs (i, j) with a[i] + b[j] == m and |i - j| >= k.
// Rethought without pb_ds order-statistic trees: since indices of `a` are pushed
// in strictly increasing order, each value bucket is already sorted, so binary
// search (`first_ge`) answers each rank query in O(log n) with flat cache-friendly lists.

let (n, k, m): (1..=1e6, 0..=n, Int) = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()

var pos = HashMap<Int, List<Int>>()
for (i, x) in a.items:
    (pos[x] ??= []).push(i)

var ans = 0
for (i, y) in b.items:
    if let xs = pos.get(m - y):
        if k == 0:
            ans += xs.len
        else:
            // FRICTION MC5: `Set` has `.rank(x) -> Int` (count of elements < x),
            // whereas sorted `List` only has `.first_ge(x) -> ?Int`, forcing
            // `?? xs.len` on every count-below / count-above query.
            let left = xs.first_ge(i - k + 1) ?? xs.len
            let right = xs.len - (xs.first_ge(i + k) ?? xs.len)
            ans += left + right

print(ans)
