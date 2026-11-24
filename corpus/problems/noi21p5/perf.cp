// Tuned: a written-out implicit treap keeps one product per node and reverses it by
// swapping its diagonal, where SeqFold keeps a forward and a backward product.
//
// Source: NOI 2021 P5 (Code), https://dmoj.ca/problem/noi21p5
// limits: 3000 ms, 1024 MB
// Problem: a string of n <= 1e5 ops W/E builds a sequence from (0, 1): W adds 1 to the last term;
//   E adds 1 to the second-to-last if the last is 1, else decrements it, appends 1, 1. Print the
//   continued fraction a0 + 1/(a1 + ...) as a b mod 998244353, then after each of q <= 1e5
//   updates: APPEND c, FLIP l r (swap W and E), REVERSE l r.
// Solution: W and E act on the fraction as the matrices [[1,0],[1,1]] and [[1,1],[0,1]], so the
//   answer is the row sums of their product. A std `SeqFold` keeps the sequence of matrices
//   with its product in both directions, so REVERSE needs no transpose trick; FLIP is a lazy
//   `Bool` tag that conjugates by the swap matrix (W <-> E), exchanging the diagonal entries and
//   the anti-diagonal ones. O(log n) expected per update.
// Bounds: n, q: 1..=1e5; l, r: 1..=n + q
// original: corpus/submissions/3927923.cpp

type Z = Mod<998_244_353>
type M2 = (Z, Z, Z, Z)  // a 2 x 2 matrix, row by row

// two residue products fit an Int, so each entry reduces once
fn dot(x: Z, y: Z, u: Z, v: Z) -> Z = Z.new(x.v * y.v + u.v * v.v)
fn mul(a: M2, b: M2) -> M2 = (
    dot(a.0, b.0, a.1, b.2), dot(a.0, b.1, a.1, b.3),
    dot(a.2, b.0, a.3, b.2), dot(a.2, b.1, a.3, b.3),
)
fn step(e: Bool) -> M2 = if e: (Z(1), Z(1), Z(0), Z(1)) else: (Z(1), Z(0), Z(1), Z(1))

let (n, q): (1..=1e5, 1..=1e5) = read()
let s: Str = read()

// An implicit treap; node 0 is the empty one. all[u] is the product over u's subtree. Flipping
// every letter swaps a product's entries end for end; reversing these step products swaps
// its diagonal, so one product per node serves both.
var lc = [0]
var rc = [0]
var pri = [0]
var sz = [0]
var east = [false]
var all = [(Z(1), Z(0), Z(0), Z(1))]
var flipped = [false]
var reversed = [false]

fn mark(u: Int, flip: Bool, rev: Bool):
    if u == 0: return
    if flip:
        east[u] = not east[u]
        let m = all[u]
        all[u] = (m.3, m.2, m.1, m.0)
        flipped[u] = not flipped[u]
    if rev:
        let m = all[u]
        all[u] = (m.3, m.1, m.2, m.0)
        reversed[u] = not reversed[u]

fn down(u: Int):
    let (flip, rev) = (flipped[u], reversed[u])
    if not flip and not rev: return
    if rev:
        let t = lc[u]
        lc[u] = rc[u]
        rc[u] = t
    mark(lc[u], flip, rev)
    mark(rc[u], flip, rev)
    flipped[u] = false
    reversed[u] = false

fn pull(u: Int):
    sz[u] = 1 + sz[lc[u]] + sz[rc[u]]
    all[u] = mul(mul(all[lc[u]], step(east[u])), all[rc[u]])

/// The first k letters of u's sequence, and the rest.
fn cut(u: Int, k: Int) -> (Int, Int):
    if u == 0: return (0, 0)
    down(u)
    if k <= sz[lc[u]]:
        let (a, b) = cut(lc[u], k)
        lc[u] = b
        pull(u)
        return (a, u)
    let (a, b) = cut(rc[u], k - sz[lc[u]] - 1)
    rc[u] = a
    pull(u)
    (u, b)

fn join(a: Int, b: Int) -> Int:
    if a == 0: return b
    if b == 0: return a
    if pri[a] > pri[b]:
        down(a)
        rc[a] = join(rc[a], b)
        pull(a)
        return a
    down(b)
    lc[b] = join(a, lc[b])
    pull(b)
    b

var root = 0
fn append(e: Bool):
    lc.push(0)
    rc.push(0)
    pri.push(rand(0, 1 << 30))
    sz.push(1)
    east.push(e)
    all.push(step(e))
    flipped.push(false)
    reversed.push(false)
    root = join(root, lc.len - 1)

for c in s: append(c == 'E')

fn report():
    let m = all[root]
    print(m.0 + m.1, m.2 + m.3)

fn span() -> Range<Int>:
    let (l, r): (1..=n + q, 1..=n + q) = read()
    l - 1..r

report()
repeat q:
    let op = read<Str>()
    if op == "APPEND":
        append(read<Str>() == "E")
    else:
        let r = span()
        let (a, rest) = cut(root, r.start)
        let (b, c) = cut(rest, r.len)
        mark(b, op == "FLIP", op != "FLIP")
        root = join(join(a, b), c)
    report()
