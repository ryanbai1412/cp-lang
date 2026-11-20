// CCC '23 S2 (Symmetric Mountains): minimum subarray asymmetry for every length 1..=n.
// Expand odd and even centers using chained `while let` optional domain stepping and
// Tropical Optional `min=` — zero INF sentinels and zero manual index bounds checks.
// original: corpus/submissions/5350805.cpp

let n: 1..=5000 = read()
domain I = 0..n
let h: [I]Int = read()

var ans: [1..=n]?Int = none
for i in I:
    for off in 0..=1:
        var (cur, j) = (0, 0)
        while let l = i - j and let r = i + (j + off):
            cur += abs(h[l] - h[r])
            ans[2 * j + 1 + off] min= cur
            j += 1

print([ans[k]! for k in 1..=n])
