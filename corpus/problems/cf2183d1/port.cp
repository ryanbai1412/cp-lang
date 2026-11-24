// Source: Codeforces 2183D1 (Tree Coloring, Easy Version),
//   https://codeforces.com/contest/2183/problem/D1
// limits: 2000 ms, 512 MB
// Problem: a tree on n <= 2e5 vertices rooted at 1, all white. An operation colours black a set
//   of white vertices with pairwise distinct depths and no two adjacent. Print the fewest
//   operations to colour the whole tree. Multitest, sum of n <= 2e5.
// Solution: a depth layer needs at least its size in operations; if all of a layer's vertices
//   share one parent p, p (one layer up) can join none of them, so it needs one more. The answer
//   is the max over layers of size + [one shared parent], which is achievable. Group the vertices
//   by depth: breadth-first order lists each layer as one run, siblings side by side. O(n).
// Bounds: n: 2..=200_000; edges.u, edges.v: 1..=n

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))

var ans = 0
var size = 0
var shared = false
let order = t.order
for (k, v) in order.items:
    let prev = if k > 0: Some(order[k - 1]) else: None
    if prev.is_none_or(u => t.depth(u) != t.depth(v)):
        size = 0
        shared = t.parent(v) != None
    elif t.parent(prev!) != t.parent(v):
        shared = false
    size += 1
    ans max= size + Int(shared)
print(ans)
