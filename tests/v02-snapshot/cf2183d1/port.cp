// Source: Codeforces 2183D1 (Tree Coloring, Easy Version),
//   https://codeforces.com/contest/2183/problem/D1
// Problem: a tree on n <= 2e5 vertices rooted at 1, all white. An operation colours black a set
//   of white vertices with pairwise distinct depths and no two adjacent. Print the fewest
//   operations to colour the whole tree. Multitest, sum of n <= 2e5.
// Solution: a depth layer needs at least its size in operations; if all of a layer's vertices
//   share one parent p, p (one layer up) can join none of them, so it needs one more. The answer
//   is the max over layers of size + [one shared parent], which is achievable. Group the vertices
//   by depth: O(n).

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted(Graph.undirected(edges), root: V(1))

var ans = 0
for (_, layer) in V.group(by: t.depth):
    let p0 = t.parent(layer[0])
    let same_parent = p0 != None and layer.all(v => t.parent(v) == p0)
    ans max= layer.len + Int(same_parent)
print(ans)
