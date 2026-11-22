// COCI 2021/22 #1 Volontiranje: a permutation of 1..=n. Find as many
// pairwise disjoint longest increasing subsequences as possible; print how
// many, their length, and the positions of each.
//
// Put every position on the level "longest increasing run ending here".
// Along one level, by position, the values fall. Grow one chain at a time
// with a stack: look at the next unused position on the level above the
// top. If it lies left of the top it can never follow anything still to
// come, so it is thrown away; if its value is smaller, the top can never be
// extended, so the top is thrown away; otherwise it goes on the stack.
// original: corpus/submissions/4086660.cpp
let n: 1..=1e6 = read()
domain Pos = 1..=n
let p: [Pos]1..=n = read()

var tails: List<Int> = []                       // smallest tail value per length
var level: List<List<Pos>> = []                 // positions per level, left to right
for i in Pos:
    let at = tails.first_ge(p[i]) ?? tails.len
    if at == tails.len:
        tails.push(p[i])
        level.push([])
    else: tails[at] = p[i]
    level[at].push(i)

let len = level.len
var next: [0..len]Int = 0                       // first unused index in each level
var chains: List<List<Pos>> = []
var chain: List<Pos> = []
while true:
    if chain.len == len:
        chains.push(chain)
        chain = []
    let h = chain.len
    if next[h] == level[h].len: break
    let cand = level[h][next[h]]
    if let top = chain.last:
        if cand < top: next[h] += 1
        elif p[cand] < p[top]: chain.pop()
        else:
            chain.push(cand)
            next[h] += 1
    else:
        chain.push(cand)
        next[h] += 1

print(chains.len, len)
for c in chains: print(c)
