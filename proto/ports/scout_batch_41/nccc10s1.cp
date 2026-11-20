// Mock CCC '22 Contest 2 S1 (nccc10s1): given n T/F answer keys of length k,
// pick a k-bit answer key maximizing the minimum Hamming distance to all n keys.
// original: corpus/submissions/4214147.cpp
let (n, k): (1..=1_000, 1..=10) = read()
domain Q = 0..k
var masks: List<Subset<Q>> = []
for _ in 0..n:
    let s: Str = read()
    var m = Subset<Q>()
    for q in Q if s[q.to_idx] == 'T': m.add(q)
    masks.push(m)
print(max(min((cand ^ m).len for m in masks)! for cand in Subset<Q>)!)
