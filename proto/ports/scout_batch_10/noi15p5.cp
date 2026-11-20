// NOI 2015 Wine Tasting: n glasses in a row, glass i has a letter s[i] and a
// score a[i]. Two glasses are "r-alike" when the r letters starting at each
// are the same. For every r in 0..n print how many pairs are r-alike and the
// largest product of scores over those pairs (0 0 if there are none).
//
// Sort the suffixes. Two glasses are r-alike exactly when every adjacent pair
// between them in sorted order shares r letters, so joining adjacent suffixes
// from the longest shared prefix down grows the groups of r-alike glasses. A
// group keeps its size and its lowest and highest score: the best product of
// a pair across two groups is at one of the four corners.
// original: corpus/submissions/3927929.cpp
use std.strings.{suffix_array, lcp_array}

let n: Int = read()
let s: Str = read()
domain Glass = 0..n
let score: [Glass]Int = read()

let letters = [c for c in s]
let sa = suffix_array(letters)
let lcp = lcp_array(letters, sa)

// Glass is reused for places in sorted order; group data lives at the root.
var groups = Dsu<Glass>.new()
var lo: [Glass]Int = [Glass] j => score[Glass.nth(sa[j.to_idx])]
var hi: [Glass]Int = [Glass] j => score[Glass.nth(sa[j.to_idx])]
var pairs = 0
var best: ?Int = none

fn join(j: Int):
    let (a, b) = (groups.find(Glass.nth(j)), groups.find(Glass.nth(j + 1)))
    pairs += groups.size(a) * groups.size(b)
    let corner = max(lo[a] * lo[b], lo[a] * hi[b], hi[a] * lo[b], hi[a] * hi[b])
    best = max(best ?? corner, corner)
    groups.unite(a, b)
    let root = groups.find(a)
    (lo[root], hi[root]) = (min(lo[a], lo[b]), max(hi[a], hi[b]))

let by_lcp = sorted([j for j in 0..n - 1], by: j => -lcp[j])
var joined = 0
var answer: List<(Int, Int)> = []
for r in (0..n).rev:
    while joined < by_lcp.len and lcp[by_lcp[joined]] >= r:
        join(by_lcp[joined])
        joined += 1
    answer.push((pairs, best ?? 0))
for (count, product) in answer.rev: print(count, product)
