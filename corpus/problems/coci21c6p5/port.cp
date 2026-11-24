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
// Bounds: n: 1..=1e5; pairs.a, pairs.b: 1..=2 * n

@multitest
let n: 1..=1e5 = read()
let s: Str = read()
domain Pos = 1..=2 * n
domain Pair = 0..n
let pairs: [Pair](a: Pos, b: Pos) = read()
fn bracket(p: Pos) -> Char = s[p.ord]

var kept: [Pos]Bool = false
var mixed_at: [Pos]?Pair = None
for p in Pair:
    let (a, b) = pairs[p]
    if bracket(a) == bracket(b): kept[if bracket(a) == '(': a else: b] = true
    else:
        mixed_at[a] = p
        kept[if bracket(a) == ')': a else: b] = true

var depth = 0
var flippable = MinHeap<(Pos, Pair)>()
for i in Pos:
    if let p = mixed_at[i]: flippable.push((pairs[p].b, p))
    if kept[i]:
        depth += if bracket(i) == '(': 1 else: -1
        if depth < 0:
            if let (_, p) = flippable.pop():
                let (a, b) = pairs[p]
                kept.swap(a, b)
                depth += if b <= i: 2 else: 1
            else:
                print(-1)
                return
if depth != 0:
    print(-1)
    return
print([Int(kept[pairs[p].b]) for p in Pair])
