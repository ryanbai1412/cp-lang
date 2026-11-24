// Source: CCC 2023 Senior S2 (Symmetric Mountains), https://dmoj.ca/problem/ccc23s2
// Problem: n <= 5000 heights 0 <= h_i <= 1e5. A crop h[l..=r] has asymmetric value the sum of
//   |h[l + i] - h[r - i]| over the pairs from the outside in. For every length 1..=n print the
//   least asymmetric value of a crop of that length.
// Solution: grow every crop from its centre: odd (i) and even (i, i + 1) centres extend one pair
//   at a time, adding |h[i - j] - h[i + j + off]|, and length 2j + 1 + off keeps the least.
//   O(n^2).
// original: corpus/submissions/5350805.cpp
// Bounds: n: 1..=5000; h: 0..=1e5
let n: 1..=5000 = read()
let h: [n]0..=1e5 = read()
domain Len = 1..=n
var least: [Len]?Int = None
for centre in 0..n:
    for off in 0..=1:
        var cost = 0
        for j in 0..=min(centre, n - 1 - centre - off):
            cost += abs(h[centre - j] - h[centre + j + off])
            least[Len(2 * j + 1 + off)] min= cost
print([least[l]! for l in Len])
