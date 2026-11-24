// Source: Codeforces 2187F1 (Al Fine, max version), https://codeforces.com/contest/2187/problem/F1
// limits: 3000 ms, 512 MB
// Problem: two DFS orders a, b of a rooted tree on vertices 0..=n (2 <= n <= 1e6; both start at
//   root 0, so only a_1..a_n and b_1..b_n are given). Print the largest depth (edges to the
//   root) of a tree that has both as DFS orders. Multitest, sum n <= 1e6.
// Solution: every subtree is an interval of both orders, so read b backwards in a's positions:
//   the common trees come from the cuts of that sequence's binary permutation tree (PermTree, O(n)
//   to build). A leaf is 1 deep; a Decr node whose right child is a leaf hangs its
//   left part one level deeper; any other node is as deep as its deeper child. O(n).
// Bounds: n: 2..=1_000_000; a, b: 1..=n
use std.tree.{PermTree, PermKind}

@!multitest
let n: 2..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()
let pos_a = a.inverse()
let t = PermTree.new([I] i => pos_a[b[I(n - 1 - i.ord)]])

fn depth<Node>(v: Node) -> Int:
    if t.kind[v] == PermKind.Leaf: return 1
    if t.kind[v] == PermKind.Decr and t.kind[t.right(v)] == PermKind.Leaf: return 1 + depth(t.left(v))
    max(depth(t.left(v)), depth(t.right(v)))
print(depth(t.root))
