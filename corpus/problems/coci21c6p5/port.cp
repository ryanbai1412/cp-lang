// Source: COCI 2021/2022 Contest 6 P5 (Superpozicija), https://dmoj.ca/problem/coci21c6p5
// limits: 1000 ms, 512 MB
// Problem: a string of 2n brackets split into n pairs of positions (a, b). Keep exactly one
//   bracket of each pair so the kept ones form a balanced sequence; print the choice (0 for a,
//   1 for b) or -1. Multitest, t, n <= 1e5, sum of n <= 1e5.
// Solution: start with as few '(' as possible: a pair of '(' keeps the first, a pair of ')' the
//   last, a mixed pair its ')'. Scan the prefix depth; a mixed pair becomes flippable once its
//   first position is passed (MinHeap keyed by its second position). When the depth goes
//   negative, flip the flippable pair that closes earliest to keep its '('; if none, or the
//   final depth is not 0, print -1. O(n log n).
// original: corpus/submissions/4531869.cpp

@multitest
let n: Int = read()
let s: Str = read()
var pairs: List<(Int, Int)> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    pairs.push((x - 1, y - 1))

var kept: [0..2 * n]Bool = false
var opens_mixed: [0..2 * n]?Int = None
for i in 0..n:
    let (x, y) = pairs[i]
    if s[x] == s[y]:
        kept[if s[x] == '(': x else: y] = true
    else:
        opens_mixed[x] = i
        kept[if s[x] == ')': x else: y] = true

var depth = 0
var flippable = MinHeap<(Int, Int)>()
for i in 0..2 * n:
    if let p = opens_mixed[i]: flippable.push((pairs[p].1, p))
    if not kept[i]: continue
    depth += if s[i] == '(': 1 else: -1
    if depth >= 0: continue
    if let (_, p) = flippable.pop():
        let (x, y) = pairs[p]
        kept.swap(x, y)
        depth += if y <= i: 2 else: 1
    else:
        print(-1)
        return
if depth != 0:
    print(-1)
    return
print([Int(kept[y]) for (_, y) in pairs])
