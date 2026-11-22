// Codeforces 2138 C1 — Maple and Tree Beauty (Easy Version)
//
// Label n tree vertices with k zeros and n - k ones to maximize the longest
// common subsequence of all root-to-leaf binary strings.
//
// Any common subsequence of length l can be achieved by making every level
// d in 0..l monochromatic (all 0 or all 1) up to the shallowest leaf depth
// `leaf_depth` (so l <= leaf_depth + 1). Level d contributes a block of
// `freq[d]` identical bits, while all vertices at depths >= l are free
// single-vertex items (weight 1 each). Thus length l is achievable iff some
// subset of {freq[0], ..., freq[l - 1]} plus at most sum_{d >= l} freq[d]
// unit items sums to k — checked in O(n) via sliding-window bounded subset
// sum over grouped item weights `ff[w]`, and binary-searched over l.

@!multitest
let (n, k): (2..=1000, 0..=1000) = read()
domain V = 1..=n
let raw_par: [2..=n]V = read()

var depth: [V]Int = 0
for i in 2..=n: depth[V(i)] = depth[raw_par[i]] + 1
var sz: [V]Int = 1
for i in (2..=n).rev: sz[raw_par[i]] += sz[V(i)]

let leaf_depth = min(depth[v] for v in V if sz[v] == 1)!
var freq: [0..n]Int = 0
for v in V: freq[depth[v]] += 1

fn is_good(l: Int) -> Bool:
    var ff: [0..=n]Int = 0
    for i in 0..l: ff[freq[i]] += 1
    for i in l..n: ff[1] += freq[i]
    var pos: [0..=n]Int = 0
    pos[0] = 1
    var npos: [0..=n]Int = 0
    for w in 1..=n if ff[w] > 0:
        for rem in 0..w:
            var ways = 0
            for v in rem..=n by w:
                ways += pos[v]
                npos[v] = Int(ways > 0)
                if v >= ff[w] * w: ways -= pos[v - ff[w] * w]
        swap(mut pos, mut npos)
    pos[k] != 0

print(bsearch_last(0..leaf_depth + 2, is_good)!)
