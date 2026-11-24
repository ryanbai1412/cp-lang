// Source: COCI '21 Contest 1 #5 (Volontiranje), https://dmoj.ca/problem/coci21c1p5
// Problem: a permutation of 1..n, n <= 1e6, whose longest increasing subsequence has length l.
//   Pick as many index-disjoint increasing subsequences of length l as possible: print their
//   count and l, then the indices of each.
// Solution: patience sorting (`tails.first_ge`) gives each index its level, the length of the
//   longest increasing subsequence ending there; within a level, values fall as indices rise.
//   Build chains greedily, level by level, from the leftmost unused candidate (`next[h]`):
//   a candidate left of the chain top is skipped for good; one with a smaller value than the
//   top means the top can never be extended, so pop it; otherwise push. Each index is pushed
//   or skipped once: O(n log n).
// original: corpus/submissions/4086660.cpp
let n: Int = read()
domain Pos = 1..=n
let p: [Pos]Int = read()

var tails: List<Int> = []
var level: List<List<Pos>> = []
for i in Pos:
    let lo = tails.first_ge(p[i]) ?? tails.len
    if lo == tails.len:
        tails.push(p[i])
        let empty: List<Pos> = []
        level.push(empty)
    else: tails[lo] = p[i]
    level[lo].push(i)

let len = level.len
var next: List<Int> = [0 for _ in 0..len]
var chains: List<List<Pos>> = []
var chain: List<Pos> = []
while true:
    if chain.len == len:
        chains.push(chain)
        chain = []
    let h = chain.len
    if next[h] == level[h].len: break
    let cand = level[h][next[h]]
    if h == 0:
        chain.push(cand)
        next[h] += 1
    else:
        let top = chain[h - 1]
        if cand < top: next[h] += 1
        elif p[cand] < p[top]: chain.pop()
        else:
            chain.push(cand)
            next[h] += 1

print(chains.len, len)
for c in chains: print(c)
