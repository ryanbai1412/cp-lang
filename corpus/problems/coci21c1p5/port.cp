// Source: COCI '21 Contest 1 #5 (Volontiranje), https://dmoj.ca/problem/coci21c1p5
// Problem: a permutation of 1..n, n <= 1e6, whose longest increasing subsequence has length l.
//   Pick as many index-disjoint increasing subsequences of length l as possible: print their
//   count and l, then the indices of each.
// Solution: patience sorting (`tails.first_ge`) gives each index its level, the length of the
//   longest increasing subsequence ending there; within a level, values fall as indices rise.
//   Build chains greedily, level by level, from the leftmost unused index of the next level:
//   one left of the chain top is thrown away for good; one with a smaller value than the top
//   means the top can never be extended, so the top is thrown away; otherwise it goes on the
//   chain. Each index is pushed or thrown away once: O(n log n).
let n: 1..=1e6 = read()
domain Pos = 1..=n
let p: [Pos]1..=n = read()

var tails: List<Int> = []
var on_level: List<List<Pos>> = []
for i in Pos:
    let at = tails.first_ge(p[i]) ?? tails.len
    if at == tails.len:
        tails.push(p[i])
        on_level.push([])
    else: tails[at] = p[i]
    on_level[at].push(i)

domain Level = 0..tails.len
let level = [Level] h => on_level[h.ord].clone()
var used = [Level] _ => 0
var chains: List<List<Pos>> = []
var chain: List<Pos> = []
while true:
    if chain.len == Level.count:
        chains.push(chain)
        chain = []
    let h = Level(chain.len)
    if used[h] == level[h].len: break
    let at = level[h][used[h]]
    if let top = chain.last:
        if at < top: used[h] += 1
        elif p[at] < p[top]: chain.pop()
        else:
            chain.push(at)
            used[h] += 1
    else:
        chain.push(at)
        used[h] += 1

print(chains.len, Level.count)
for c in chains: print(c)
