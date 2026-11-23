// Source: NOI 2015 Day 2 P2 (NOI '15 P5), Cocktail Party (Wine Tasting),
//   https://dmoj.ca/problem/noi15p5
// Problem: a string s of n lowercase labels (n <= 3e5) and scores a_i (|a_i| <= 1e9). Cups p != q
//   are r-similar if the length-r substrings starting at p and q are equal (all pairs are
//   0-similar). For each r = 0..n-1 print the number of r-similar pairs and the largest
//   a_p * a_q over them (0 0 if none).
// Solution: Build the suffix array and LCP array. Cups are r-similar iff every adjacent pair of
//   suffixes between them in sorted order has LCP >= r, so for r from n-1 down to 0 unite
//   adjacent suffixes (sorted by LCP descending) in a DSU. Each group keeps its size, lowest
//   and highest score; a merge adds size(a) * size(b) pairs and the best of the four corner
//   products. O(n log n).
// original: corpus/submissions/3927929.cpp

let n: Int = read()
let s: Str = read()
domain Glass = 0..n
let score: [Glass]Int = read()

let letters = s
let suffixes = SuffixArray(letters)
let (sa, lcp) = (suffixes.sa, suffixes.lcp)

// Glass is reused for places in sorted order; group data lives at the root.
var groups = Dsu<Glass>.new()
var lo: [Glass]Int = [Glass] j => score[Glass.nth(sa[j.to_idx])]
var hi: [Glass]Int = [Glass] j => score[Glass.nth(sa[j.to_idx])]
var pairs = 0
var best: ?Int = None

fn join(j: Int):
    let (a, b) = (groups.find(Glass.nth(j)), groups.find(Glass.nth(j + 1)))
    pairs += groups.size(a) * groups.size(b)
    let corner = max(lo[a] * lo[b], lo[a] * hi[b], hi[a] * lo[b], hi[a] * hi[b])
    best = max(best ?? corner, corner)
    groups.unite(a, b)
    let root = groups.find(a)
    (lo[root], hi[root]) = (min(lo[a], lo[b]), max(hi[a], hi[b]))

let by_lcp = (0..n - 1).sorted(by: j => -lcp[j])
var joined = 0
var answer: List<(Int, Int)> = []
for r in (0..n).rev:
    while joined < by_lcp.len and lcp[by_lcp[joined]] >= r:
        join(by_lcp[joined])
        joined += 1
    answer.push((pairs, best ?? 0))
for (count, product) in answer.rev: print(count, product)
