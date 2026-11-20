// Mock CCC '23 Contest 1 J5 — XOR-Prefix Range Maximum Trie (CP Lang v2 spec)
// original: corpus/submissions/5333196.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Given array a[0..n) and q queries (m, k), answer max(a[i] for 0 <= i < n with (i ^ m) <= k).
// Rethought as an iterative complete binary trie sized dynamically from `bit_len`:
// O(2^bits) build in 2 lines and O(bits) non-recursive bitwise descent per query.

let (n, q): (1..=1e6, 1..=1e6) = read()
let a: [0..n]Int = read()
let qs: [0..q](m: Int, k: Int) = read()

let hi = max(n - 1, max(max(m, k) for (m, k) in qs) ?? 0)
let bits = max(1, bit_len(hi))
let sz = 1 << bits

// Complete binary tree with leaves at [sz..2*sz); unused tail leaves stay 0.
var tree: [1..(2 * sz)]Int = 0
for i in 0..n: tree[sz + i] = a[i]
for o in (1..sz).rev: tree[o] = max(tree[2 * o], tree[2 * o + 1])

fn query(m: Int, k: Int) -> Int:
    var (o, ans) = (1, 0)
    for d in (0..bits).rev:
        let bit_m = (m >> d) & 1
        let bit_k = (k >> d) & 1
        let chi = bit_m ^ bit_k
        // When bit_k == 1, the sibling child has bit(i ^ m, d) == 0 < bit_k,
        // so its entire subtree satisfies (i ^ m) < k.
        if bit_k == 1: ans max= tree[2 * o + (1 - chi)]
        o = 2 * o + chi
    max(ans, tree[o])

for (m, k) in qs:
    print(query(m, k))
