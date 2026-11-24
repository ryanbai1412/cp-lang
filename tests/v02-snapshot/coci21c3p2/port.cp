// Source: COCI '21 Contest 3 #2, Cijanobakterije, https://dmoj.ca/problem/coci21c3p2
// Problem: a forest on n <= 1e5 vertices with m < n edges. Joining trees by new edges, print
//   the most vertices a single path (chain) can have.
// Solution: link the trees' diameters end to end, so the answer is the sum of every tree's
//   diameter counted in vertices. One DFS per tree keeps the deepest branch below each vertex
//   and the best path through it. O(n).
// original: corpus/submissions/4196157.cpp
let n: Int = read()
let m: Int = read()
domain V = 1..=n
var g: [V]List<V> = []
for _ in 0..m:
    let u: V = read()
    let v: V = read()
    g[u].push(v)
    g[v].push(u)

var seen: [V]Bool = false
var longest = 0
fn down(u: V, parent: ?V) -> Int:
    seen[u] = true
    var deep = 0
    for v in g[u]:
        if parent == None or v != parent!:
            let d = down(v, u)
            longest max= deep + d + 1
            deep max= d
    deep + 1

var total = 0
for root in V:
    if not seen[root]:
        longest = 1
        down(root, None)
        total += longest
print(total)
