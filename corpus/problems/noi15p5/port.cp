// Source: NOI 2015 Day 2 P2 (NOI '15 P5), Cocktail Party (Wine Tasting),
//   https://dmoj.ca/problem/noi15p5
// Problem: a string s of n lowercase labels (n <= 3e5) and scores a_i (|a_i| <= 1e9). Cups p != q
//   are r-similar if the length-r substrings starting at p and q are equal (all pairs are
//   0-similar). For each r = 0..n-1 print the number of r-similar pairs and the largest
//   a_p * a_q over them (0 0 if none).
// Solution: Build the suffix array and LCP array. Cups are r-similar iff every adjacent pair of
//   suffixes between them in sorted order has LCP >= r, so for r from n-1 down to 0 unite
//   adjacent suffixes whose LCP is r in a DSU. Each group keeps its size, lowest
//   and highest score; a merge adds size(a) * size(b) pairs and the best of the four corner
//   products. O(n log n).
// original: corpus/submissions/3927929.cpp

let n: 1..=3e5 = read()
let s: Str = read()
domain Glass = 0..n
domain Rank = 0..n  // places in suffix order
let score: [Glass]-1e9..=1e9 = read()
let suffixes = SuffixArray(s)
let ranked = [Rank] k => score[Glass(suffixes.sa[k.ord])]

// each group of adjacent ranks keeps its lowest and highest score at its root
var groups = Dsu<Rank>.new()
var (lo, hi) = (ranked, ranked)
var pairs = 0
var best: ?Int = None

fn join(a0: Rank, b0: Rank):
    let (a, b) = (groups.find(a0), groups.find(b0))
    pairs += groups.size(a) * groups.size(b)
    let corner = max(lo[a] * lo[b], lo[a] * hi[b], hi[a] * lo[b], hi[a] * hi[b])
    best = max(best ?? corner, corner)
    groups.unite(a, b)
    let root = groups.find(a)
    (lo[root], hi[root]) = (min(lo[a], lo[b]), max(hi[a], hi[b]))

// lcp[j] is the common prefix of ranks j and j + 1
var meet_at: [0..n]List<Int> = []
for j in 0..n - 1: meet_at[suffixes.lcp[j]].push(j)
var answer: List<(Int, Int)> = []
for r in (0..n).rev:
    for j in meet_at[r]: join(Rank(j), Rank(j + 1))
    answer.push((pairs, best ?? 0))
for (count, product) in answer.rev: print(count, product)
