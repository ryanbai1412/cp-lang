@!multitest
let (n, k): (2..=2e5, 1..=1500) = read()
let a: [0..n]1..=1e9 = read()
var best = min(n, k + 2)
// ends[p % w]: chains whose last kept element is p, as (d, left, kept), sorted by d
let w = k + 2
var ends: List<List<(Int, Int, Int)>> = [[] for _ in 0..w]
for j in 1..n:
    var cands: List<(Int, Int)> = []
    for g in 1..=min(k + 1, j):
        let diff = a[j] - a[j - g]
        if diff % g == 0:
            cands.push((diff / g, g))
    cands.sort()
    var mine: List<(Int, Int, Int)> = []
    for t in 0..cands.len:
        let (d, g) = cands[t]
        if t > 0 and cands[t - 1].0 == d:
            continue
        let p = j - g
        let at = ends[p % w].bsearch(e => e.0 >= d)
        var (left, kept) = (p, 1)
        if at < ends[p % w].len and ends[p % w][at].0 == d:
            (left, kept) = (ends[p % w][at].1, ends[p % w][at].2)
        kept += 1
        let c = a[j] - j * d
        while (j - left + 1) - kept > k:
            left += 1
            while a[left] - left * d != c:
                left += 1
            kept -= 1
        mine.push((d, left, kept))
        best = max(best, min(n, kept + k))
    ends[j % w] = mine
print(best)
