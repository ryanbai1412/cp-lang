// Volontiranje, in the forms proto/cpc builds today. See ../coci21c1p5.cp.
// `tails.first_ge` is a hand binary search.
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
