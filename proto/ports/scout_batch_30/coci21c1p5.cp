// COCI 2021/2022 Contest #1 P5 Volontiranje: given a permutation of 1..=n,
// find the maximum number of pairwise-disjoint longest increasing subsequences
// and output one valid collection of 1-based index sequences.
// Patience-sort into LIS layers 1..=L. Pack all layers contiguously into one
// flat min-SegTree<(val, flat_idx)> (avoiding List<SegTree<D, T>> across
// per-layer domains). Repeatedly extract a lexicographically earliest valid
// LIS via backtracking DFS, deleting every visited node once.
// original: corpus/submissions/4086660.cpp
let n: 1..=1000000 = read()
let arr: [0..n]Int = read()

const INF = 1000000000
var tails: [0..=n]Int = INF
tails[0] = -INF
var layers: [0..=n]List<Int> = []
var mlen = 0

for i in 0..n:
    let x = arr[i]
    var lo = 0
    var hi = mlen + 1
    while lo < hi:
        let mid = (lo + hi) / 2
        if tails[mid] < x: lo = mid + 1
        else: hi = mid
    tails[lo] = x
    if lo > mlen: mlen = lo
    layers[lo].push(i)

// Pack all n elements by layer into a single flat domain Flat = 0..n so one
// SegTree serves every layer via slice [off[k], off[k + 1]).
domain Flat = 0..n
var off: [0..=mlen + 1]Int = 0
for k in 1..=mlen:
    off[k + 1] = off[k] + layers[k].len

var flat_time: [0..n]Int = 0
var st = SegTree<Flat, (Int, Int)>((INF, -1), (a, b) => min(a, b))
for k in 1..=mlen:
    let base = off[k]
    for j in 0..layers[k].len:
        let pos = layers[k][j]
        flat_time[base + j] = pos
        st.set(Flat(base + j), (arr[pos], base + j))

fn upper_time(k: Int, ptime: Int) -> Int:
    let base = off[k]
    var lo = 0
    var hi = off[k + 1] - base
    while lo < hi:
        let mid = (lo + hi) / 2
        if flat_time[base + mid] <= ptime: lo = mid + 1
        else: hi = mid
    lo

var cur: List<Int> = []

fn dfs(k: Int, pval: Int, ptime: Int) -> Bool:
    if k == 0: return true
    while true:
        let cnt = upper_time(k, ptime)
        if cnt == 0: return false
        let base = off[k]
        let best = st.query(Flat(base), Flat(base + cnt - 1))
        if best.0 >= pval: return false
        let fidx = best.1
        st.set(Flat(fidx), (INF, -1))
        let pos = flat_time[fidx]
        cur.push(pos)
        if dfs(k - 1, best.0, pos): return true
        let _ = cur.pop()
    false

var ans: List<List<Int>> = []
while true:
    cur = []
    if not dfs(mlen, INF, INF): break
    var seq: List<Int> = []
    for i in (0..cur.len).rev:
        seq.push(cur[i] + 1)
    ans.push(seq)

print(ans.len, mlen)
for seq in ans:
    print(seq)
