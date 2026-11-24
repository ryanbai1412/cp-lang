// Source: AtCoder Educational DP Contest P (Independent Set),
//   https://atcoder.jp/contests/dp/tasks/dp_p
// limits: 2000 ms, 1024 MB
// Problem: a tree on n <= 1e5 vertices. Count the ways to paint each vertex white or black
//   with no two adjacent vertices black, modulo 1e9 + 7.
// Solution: tree DP rooted at vertex 1: white[v] / black[v] count paintings of v's subtree
//   with v white / black; a child c multiplies white[p] by white[c] + black[c] and black[p]
//   by white[c], children before parents in reverse visit order. O(n).
type Z = Mod<1_000_000_007>
let n: 1..=1e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first!)

// paintings of v's subtree with v white / black
var white: [V]Z = 1
var black: [V]Z = 1
for v in tree.at.rev:
    if let p = tree.parent(v):
        white[p] *= white[v] + black[v]
        black[p] *= white[v]
print(white[V.first!] + black[V.first!])
