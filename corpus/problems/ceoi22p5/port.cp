// Source: CEOI 2022 P5 (Measures), https://dmoj.ca/problem/ceoi22p5
// Problem: N people on a line, then M more added one at a time (N, M <= 2e5; D and positions
//   <= 1e9). After each addition print the least time for all, moving at speed 1, to stand
//   pairwise at least D apart, as an exact decimal without trailing zeros.
// Solution: in sorted order the answer is max(0, max over i < j of ((j - i) D - (x_j - x_i)) / 2).
//   Rank all N + M people by position offline; a segment tree over ranks keeps, for the people
//   present, the count, max(r D - x), max(x + r' D) (r, r' = how many lie left / right in the
//   node) and the best pair inside, merged by shifting with counts. Positions are doubled so
//   the halves stay integral. O((N + M) log(N + M)).
// original: corpus/submissions/4733784.cpp
let (n, m, gap1): (Int, Int, Int) = read()
let total = n + m
let gap = 2 * gap1  // work in half units
let neg = -1000000000000000000
var xs: List<Int> = []
for _ in 0..total:
    let x: Int = read()
    xs.push(2 * x)

// rank[i]: leaf of person i when everyone is sorted by position.
let order = (0..total).sorted(by: i => (xs[i], i))
var rank = List.filled(total, 0)
for r in 0..total: rank[order[r]] = r

// Segment tree over ranks; a node summarises the people present in its span:
// siz, max(r * gap - x), max(x - r' * gap) and the best answer inside.
var siz = List.filled(4 * total, 0)
var pref = List.filled(4 * total, neg)
var suff = List.filled(4 * total, neg)
var best = List.filled(4 * total, 0)

fn insert(o: Int, l: Int, r: Int, at: Int, x: Int):
    if l == r:
        siz[o] = 1
        pref[o] = -x
        suff[o] = x
        return
    let mid = (l + r) / 2
    let (a, b) = (2 * o, 2 * o + 1)
    if at <= mid: insert(a, l, mid, at, x)
    else: insert(b, mid + 1, r, at, x)
    siz[o] = siz[a] + siz[b]
    pref[o] = max(pref[a], pref[b] + siz[a] * gap)
    suff[o] = max(suff[b], suff[a] + siz[b] * gap)
    best[o] = max(best[a], best[b], (pref[b] + suff[a] + gap) / 2)

for i in 0..total:
    insert(1, 0, total - 1, rank[i], xs[i])
    if i >= n:
        let t = best[1]
        print(if t % 2 == 1: "{t / 2}.5" else: "{t / 2}")
