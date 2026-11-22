// BOI 2017 Political Development, as it builds today: one set of pairs, a lazy
// heap of (degree, person), and subsets as bit masks (K9, K18).
// original: corpus/submissions/4456179.cpp
let n: 1..=5e4 = read()
let k: 1..=10 = read()
domain P = 0..n
let lists: [P]List<P> = read()

var pals: Set<(P, P)> = []
for p in P:
    for q in lists[p]: pals.insert((p, q))
var deg: [P]Int = [P] p => lists[p].len
var gone: [P]Bool = false
var heap = MinHeap<(Int, P)>()
for p in P: heap.push((deg[p], p))
var best = 0
while let (d, u) = heap.pop():
    if gone[u] or d != deg[u]: continue
    gone[u] = true
    for v in lists[u]:
        deg[v] -= 1
        heap.push((deg[v], v))
    let near = [v for v in lists[u] if not gone[v]]
    for mask in 0..(1 << near.len):
        let group = [near[i] for i in 0..near.len if ((mask >> i) & 1) == 1]
        if all((a, b) in pals for a in group for b in group if a != b): best max= group.len + 1
print(best)
