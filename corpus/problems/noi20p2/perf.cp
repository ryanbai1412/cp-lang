// Tuned: the accepted C++'s pool of lazy affine segment trees over depths, merged by
// multiplying, in plain lists: a node holds a pending x => x * mu + ad, so a vertex's own state
// is one node instead of a std MergeTrees root-to-leaf path, and the merge hook is inline.
//
// Source: NOI 2020 P2 (Destiny), https://dmoj.ca/problem/noi20p2
// limits: 2000 ms, 1024 MB
// Problem: a tree on n <= 5e5 vertices rooted at 1 and m <= 5e5 pairs (u, v), u an ancestor
//   of v. Count the 0/1 labellings of the edges where every pair's path has an edge labelled 1,
//   mod 998244353.
// Solution: dp[v][d] counts labellings of v's subtree whose deepest unsatisfied pair has its
//   top at depth d (0: none). Merging child c: edge (v, c) = 1 moves all of c's states with
//   d <= depth(v) to 0; then new[d] = a[d] * B(<= d) + b[d] * A(< d), a max-convolution done
//   by a std `MergeTrees` merge, whose hook sees both values and both prefix sums. O((n + m) log n).
// original: corpus/submissions/5655455.cpp
// Bounds: n: 1..=5e5; edges.u, edges.v: 1..=n; m: 1..=5e5; paths.u, paths.v: 1..=n

type Z = Mod<998_244_353>

let n: 1..=5e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let m: 1..=5e5 = read()
domain P = 0..m
let paths: [P](u: V, v: V) = read()

let tree = Rooted(Graph<V, E>.undirected(edges), V.first!)
var top: [V]Int = -1  // the depth of the deepest pair top over v, -1 if none
for p in P: top[paths[p].v] max= tree.depth(paths[p].u)

// segment trees over depths 0..=n; node 0 is the all-zero tree
type Node = 0..=1 << 26  // a pool index, stored narrow
var (lc, rc): (List<Node>, List<Node>) = ([0], [0])
var (mu, ad) = ([Z(0)], [Z(0)])
fn alloc() -> Node:
    lc.push(0)
    rc.push(0)
    mu.push(Z(1))
    ad.push(Z(0))
    lc.len - 1
fn leaf(o: Int) -> Bool = lc[o] == 0 and rc[o] == 0

fn get(root: Int, at: Int) -> Z:
    var (o, lo, hi, k, b) = (root, 0, n, Z(1), Z(0))
    while o != 0:
        (k, b) = (mu[o] * k, ad[o] * k + b)  // a node's map runs before its ancestors'
        let mid = (lo + hi) / 2
        if at <= mid:
            (o, hi) = (lc[o], mid)
        else:
            (o, lo) = (rc[o], mid + 1)
    b

fn push_child(o: Int, c0: Int) -> Int:
    let c = if c0 == 0: alloc() else: c0
    (mu[c], ad[c]) = (mu[c] * mu[o], ad[c] * mu[o] + ad[o])
    c
fn push(o: Int):
    if mu[o] == Z(1) and ad[o] == Z(0): return
    let l = push_child(o, lc[o])
    lc[o] = l
    let r = push_child(o, rc[o])
    rc[o] = r
    (mu[o], ad[o]) = (Z(1), Z(0))

fn mult(x: Int, y: Int) -> Int:
    let (a, b) = if leaf(y): (y, x) else: (x, y)
    if leaf(a):
        let k = ad[a]  // a leaf is the constant its map gives 0
        (mu[b], ad[b]) = (mu[b] * k, ad[b] * k)
        return b
    push(a)
    push(b)
    let l = mult(lc[a], lc[b])
    lc[a] = l
    let r = mult(rc[a], rc[b])
    rc[a] = r
    a

// zero depths 0..=upto
fn clear(o: Int, lo: Int, hi: Int, upto: Int) -> Int:
    if o == 0 or hi <= upto: return 0
    push(o)
    let mid = (lo + hi) / 2
    let l = clear(lc[o], lo, mid, upto)
    lc[o] = l
    if mid < upto:
        let r = clear(rc[o], mid + 1, hi, upto)
        rc[o] = r
    o

var dp: [V]Int = 0
var answer = Z(1)
for u in tree.at.rev:
    if u == V.first!:
        for c in tree.children(u): answer *= get(dp[c], 0)
        continue
    var lifted = Z(1)
    var own = alloc()
    ad[own] += Z(1)
    for c in tree.children(u):
        lifted *= get(dp[c], tree.depth(u))
        own = mult(own, dp[c])
    if top[u] >= 0: own = clear(own, 0, n, top[u])
    ad[own] += lifted
    dp[u] = own
print(answer)
