// BOI 2018 P2, DNA: a string of n symbols over an alphabet of k, and r
// requirements "at least q copies of symbol b". Print the length of the
// shortest substring that meets all of them, or "impossible".
//
// Two pointers: grow the window to the right; while every requirement is
// met, record it and shrink from the left. `unmet` counts the required
// symbols the window is still short of, so "all met" is one comparison.
// original: corpus/submissions/4456171.cpp
let (n, k, r): (Int, Int, Int) = read()
domain Pos = 0..n
domain Sym = 0..k
let dna: [Pos]Sym = read()
var need: [Sym]Int = 0
for _ in 0..r:
    let (b, q): (Sym, Int) = read()
    need[b] max= q

var have: [Sym]Int = 0
// WISH C1: `count(need[s] > 0 for s in Sym)` is on the cheat sheet, not in the prelude.
var unmet = sum(1 for s in Sym if need[s] > 0)
var lo = Pos.cuts.first
var best: ?Int = none
for hi in Pos:
    let s = dna[hi]
    have[s] += 1
    if have[s] == need[s]: unmet -= 1
    while unmet == 0:
        best = min(best ?? n, (lo..hi.r).len)
        let gone = dna[lo.right!]
        if have[gone] == need[gone]: unmet += 1
        have[gone] -= 1
        lo = lo.next!
print(best ?? "impossible")
