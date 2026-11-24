// Source: BOI 2017 Day 1 (Political Development), https://dmoj.ca/problem/btoi17p1
// limits: 1000 ms, 1024 MB
// Problem: n <= 5e4 politicians, each with the list of those they disagree with (symmetric),
//   and K <= 10 such that every non-empty group has a member disagreeing with < K others in
//   it. Print the size of the largest group in which every two disagree (a largest clique).
// Solution: the graph is (K-1)-degenerate, so a vertex of least remaining degree has < K
//   remaining neighbours. Remove vertices in that order with a lazy MinHeap of (degree,
//   person); a clique's first removed vertex sees the rest among its remaining neighbours,
//   so try every subset of those as a Subset, a clique when its first member
//   knows the rest and the rest is a clique: O(n 2^K K + m log m).
// original: corpus/submissions/4456179.cpp
// Bounds: n: 1..=5e4; k: 1..=10; lists: 0..n
let n: 1..=5e4 = read()
let k: 1..=10 = read()
domain P = 0..n
let lists: [P]List<P> = read()

// each list sorted, so a pair is a binary search
let foes = [P] p => lists[p].sorted()
fn foes_of(p: P, q: P) -> Bool = foes[p].first_ge(q).map(i => foes[p][i] == q) ?? false
var deg: [P]Int = [P] p => lists[p].len
var gone: [P]Bool = false
var heap = MinHeap<(Int, P)>()
for p in P: heap.push((deg[p], p))
var best = 1
while let (d, u) = heap.pop():
    if gone[u] or d != deg[u]: continue
    gone[u] = true
    for v in lists[u]:
        deg[v] -= 1
        heap.push((deg[v], v))
    // u's remaining neighbours: fewer than k of them, so every subset fits a word
    let near = [v for v in lists[u] if not gone[v]]
    domain Near = 0..near.len
    var mates: [Near]Subset<Near> = Subset<Near>.empty
    for i in Near:
        for j in Near if foes_of(near[i.ord], near[j.ord]): mates[i] = mates[i].with(j)
    var clique: [Near.subsets]Bool = true
    for s in Subset<Near> if let i = s.first():
        let rest = s.without(i)
        clique[s] = clique[rest] and (rest - mates[i]).is_empty
        if clique[s]: best max= s.len + 1
print(best)
