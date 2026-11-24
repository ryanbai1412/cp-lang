// Tuned: the accepted C++'s pool of lazy affine segment trees over depths, merged by
// multiplying, in one reserved pool: a node holds a pending x => x * mu + ad, so a vertex's
// own state is one node instead of a std MergeTrees root-to-leaf path, and the merge hook is inline.
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

let tree = Rooted(Graph.undirected(edges), V.first!)
var top: [V]Int = -1  // the depth of the deepest pair top over v, -1 if none
for p in P: top[paths[p].v] max= tree.depth(paths[p].u)

// segment trees over depths 0..=n; node 0 is the all-zero tree
type Node = 0..=1 << 26  // a pool index, stored narrow
struct Tree:  // children, and the pending x => x * mu + ad
    var lc: Node
    var rc: Node
    var mu: Z
    var ad: Z
var pool = [Tree(0, 0, Z(0), Z(0))]
pool.reserve(20 * n)  // the accepted C++'s pool size; a vertex adds O(log n) nodes
fn alloc() -> Node:
    pool.push(Tree(0, 0, Z(1), Z(0)))
    pool.len - 1
fn leaf(o: Int) -> Bool = pool[o].lc == 0 and pool[o].rc == 0

fn get(root: Int, at: Int) -> Z:
    var (o, lo, hi, k, b) = (root, 0, n, Z(1), Z(0))
    while o != 0:
        (k, b) = (pool[o].mu * k, pool[o].ad * k + b)  // a node's map runs before its ancestors'
        let mid = (lo + hi) / 2
        if at <= mid:
            (o, hi) = (pool[o].lc, mid)
        else:
            (o, lo) = (pool[o].rc, mid + 1)
    b

fn push_child(o: Int, c0: Int) -> Int:
    let c = if c0 == 0: alloc() else: c0
    (pool[c].mu, pool[c].ad) = (pool[c].mu * pool[o].mu, pool[c].ad * pool[o].mu + pool[o].ad)
    c
fn push(o: Int):
    if pool[o].mu == Z(1) and pool[o].ad == Z(0): return
    let l = push_child(o, pool[o].lc)
    pool[o].lc = l
    let r = push_child(o, pool[o].rc)
    pool[o].rc = r
    (pool[o].mu, pool[o].ad) = (Z(1), Z(0))

fn mult(x: Int, y: Int) -> Int:
    let (a, b) = if leaf(y): (y, x) else: (x, y)
    if leaf(a):
        let k = pool[a].ad  // a leaf is the constant its map gives 0
        (pool[b].mu, pool[b].ad) = (pool[b].mu * k, pool[b].ad * k)
        return b
    push(a)
    push(b)
    let l = mult(pool[a].lc, pool[b].lc)
    pool[a].lc = l
    let r = mult(pool[a].rc, pool[b].rc)
    pool[a].rc = r
    a

// zero depths 0..=upto
fn clear(o: Int, lo: Int, hi: Int, upto: Int) -> Int:
    if o == 0 or hi <= upto: return 0
    push(o)
    let mid = (lo + hi) / 2
    let l = clear(pool[o].lc, lo, mid, upto)
    pool[o].lc = l
    if mid < upto:
        let r = clear(pool[o].rc, mid + 1, hi, upto)
        pool[o].rc = r
    o

var dp: [V]Int = 0
var answer = Z(1)
for u in tree.at.rev:
    if u == V.first!:
        for c in tree.children(u): answer *= get(dp[c], 0)
        continue
    var lifted = Z(1)
    var own = alloc()
    pool[own].ad += Z(1)
    for c in tree.children(u):
        lifted *= get(dp[c], tree.depth(u))
        own = mult(own, dp[c])
    if top[u] >= 0: own = clear(own, 0, n, top[u])
    pool[own].ad += lifted
    dp[u] = own
print(answer)
