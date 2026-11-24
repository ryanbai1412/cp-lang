// Source: BOI 2018 Day 1 (DNA), https://dmoj.ca/problem/btoi18p2
// limits: 500 ms, 1024 MB
// Problem: a string of N <= 2e5 symbols over an alphabet of K <= N, and R <= K requirements "at
//   least Q copies of symbol B" (distinct B). Print the length of the shortest substring meeting
//   all of them, or "impossible".
// Solution: two pointers: grow the window to the right; while every requirement is met, record
//   its length and shrink from the left. `unmet` counts required symbols still short in the
//   window, so "all met" is one comparison. O(N + K).
// original: corpus/submissions/4456171.cpp
// Bounds: n: 1..=2e5; k: 1..=n; r: 1..=k; dna: 0..k; wants.b: 0..k; wants.q: 1..=n
let n: 1..=2e5 = read()
let k: 1..=n = read()
let r: 1..=k = read()
domain Pos = 0..n
domain Sym = 0..k
let dna: [Pos]Sym = read()
let wants: [r](b: Sym, q: 1..=n) = read()
var need: [Sym]Int = 0
for w in wants: need[w.b] = w.q

var have: [Sym]Int = 0
var unmet = count(need[s] > 0 for s in Sym)
var lo = Pos.cuts.first!
var best: ?Int = None
for hi in Pos:
    let s = dna[hi]
    have[s] += 1
    if have[s] == need[s]: unmet -= 1
    while unmet == 0:
        best min= Some((lo..hi.r).len)
        let gone = dna[lo.right!]
        if have[gone] == need[gone]: unmet += 1
        have[gone] -= 1
        lo = lo.next!
if let b = best: print(b) else: print("impossible")
