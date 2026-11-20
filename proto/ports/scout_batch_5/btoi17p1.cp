// BOI 2017 Political Development: everyone lists who they disagree with
// (mutual). Among any group somebody disagrees with fewer than k of them.
// Print the size of the largest group that all disagree with each other.
// original: corpus/submissions/4456179.cpp
let n: 1..=5e4 = read()
let k: 1..=10 = read()
domain P = 0..n
let lists: [P]List<P> = read()                                      // K4: generated without the lengths when `gen:` is used

// Remove the person with the fewest remaining opponents, again and again.
// The largest group whose first removed member is u lies inside u's
// remaining opponents, fewer than k of them: try every subset.
let foes = [P] p => Set(lists[p])                                   // WISH G5
var alive = Set(P)                                                  // WISH G5
var by_degree = MinHeap.by(P, p => (foes[p] & alive).len)           // WISH K9: a heap that follows its key
var best = 0
while let u = by_degree.pop():
    alive.remove(u)
    let near = foes[u] & alive                                      // WISH B6 (batch 4): set algebra
    best max= 1 + max(group.len for group in near.subsets if all(b in foes[a] for (a, b) in group.pairs))   // WISH B6, K18
print(best)
