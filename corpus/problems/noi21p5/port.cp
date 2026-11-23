// Source: NOI 2021 P5 (Code), https://dmoj.ca/problem/noi21p5
// Problem: a string of n <= 1e5 ops W/E builds a sequence from (0, 1): W adds 1 to the last term;
//   E adds 1 to the second-to-last if the last is 1, else decrements it, appends 1, 1. Print the
//   continued fraction a0 + 1/(a1 + ...) as a b mod 998244353, then after each of q <= 1e5
//   updates: APPEND c, FLIP l r (swap W and E), REVERSE l r.
// Solution: W and E act on the fraction as the matrices [[1,0],[1,1]] and [[1,1],[0,1]], so the
//   answer is the row sums of their product. An implicit treap keeps each subtree's product mod p
//   with lazy flip and reverse tags: flipping swaps the anti-diagonal and the diagonal, and since
//   E is W transposed, reversing just swaps the diagonal. Updates split out l..r, tag and merge
//   back, O(log n) expected each.
// original: corpus/submissions/3927923.cpp

struct Node:
    var is_e: Bool
    var sz: Int
    pri: Int
    var flip: Bool
    var rev: Bool
    var a00: Int
    var a01: Int
    var a10: Int
    var a11: Int
    var lc: Int
    var rc: Int

const MODV = 998_244_353
var g_nodes: List<Node> = []
var g_seed = 141234567

fn next_rand() -> Int:
    g_seed = (g_seed * 1103515245 + 12345) % 2147483647
    return g_seed

fn mat_mul(
    a00: Int,
    a01: Int,
    a10: Int,
    a11: Int,
    b00: Int,
    b01: Int,
    b10: Int,
    b11: Int,
) -> (Int, Int, Int, Int):
    return (
        (a00 * b00 + a01 * b10) % MODV,
        (a00 * b01 + a01 * b11) % MODV,
        (a10 * b00 + a11 * b10) % MODV,
        (a10 * b01 + a11 * b11) % MODV,
    )

fn make_node(is_e: Bool) -> Int:
    let id = g_nodes.len
    let m01 = Int(is_e)
    let m10 = if is_e: 0 else: 1
    g_nodes.push(Node(is_e, 1, next_rand(), false, false, 1, m01, m10, 1, -1, -1))
    return id

fn size_of(u: Int) -> Int = if u == -1: 0 else: g_nodes[u].sz

fn pull(u: Int):
    var sz = 1
    var m00 = 1
    var m01 = Int(g_nodes[u].is_e)
    var m10 = if g_nodes[u].is_e: 0 else: 1
    var m11 = 1
    let l = g_nodes[u].lc
    if l != -1:
        sz += g_nodes[l].sz
        let p = mat_mul(
            g_nodes[l].a00,
            g_nodes[l].a01,
            g_nodes[l].a10,
            g_nodes[l].a11,
            m00,
            m01,
            m10,
            m11,
        )
        m00 = p.0
        m01 = p.1
        m10 = p.2
        m11 = p.3
    let r = g_nodes[u].rc
    if r != -1:
        sz += g_nodes[r].sz
        let p = mat_mul(
            m00,
            m01,
            m10,
            m11,
            g_nodes[r].a00,
            g_nodes[r].a01,
            g_nodes[r].a10,
            g_nodes[r].a11,
        )
        m00 = p.0
        m01 = p.1
        m10 = p.2
        m11 = p.3
    if g_nodes[u].flip:
        swap(mut m00, mut m11)
        swap(mut m01, mut m10)
    if g_nodes[u].rev:
        swap(mut m00, mut m11)
    g_nodes[u].sz = sz
    g_nodes[u].a00 = m00
    g_nodes[u].a01 = m01
    g_nodes[u].a10 = m10
    g_nodes[u].a11 = m11

fn push(u: Int):
    let fl = g_nodes[u].flip
    let rv = g_nodes[u].rev
    if fl: g_nodes[u].is_e = not g_nodes[u].is_e
    if rv:
        swap(mut g_nodes[u].lc, mut g_nodes[u].rc)
    let lc = g_nodes[u].lc
    if lc != -1:
        g_nodes[lc].flip = g_nodes[lc].flip != fl
        g_nodes[lc].rev = g_nodes[lc].rev != rv
        pull(lc)
    let rc = g_nodes[u].rc
    if rc != -1:
        g_nodes[rc].flip = g_nodes[rc].flip != fl
        g_nodes[rc].rev = g_nodes[rc].rev != rv
        pull(rc)
    g_nodes[u].flip = false
    g_nodes[u].rev = false

fn split(u: Int, k: Int) -> (Int, Int):
    if u == -1: return (-1, -1)
    push(u)
    let lsz = size_of(g_nodes[u].lc)
    if k <= lsz:
        let p = split(g_nodes[u].lc, k)
        g_nodes[u].lc = p.1
        pull(u)
        return (p.0, u)
    let p = split(g_nodes[u].rc, k - lsz - 1)
    g_nodes[u].rc = p.0
    pull(u)
    return (u, p.1)

fn merge(l: Int, r: Int) -> Int:
    if l == -1: return r
    if r == -1: return l
    if g_nodes[l].pri < g_nodes[r].pri:
        push(l)
        g_nodes[l].rc = merge(g_nodes[l].rc, r)
        pull(l)
        return l
    push(r)
    g_nodes[r].lc = merge(l, g_nodes[r].lc)
    pull(r)
    return r

fn report(u: Int):
    let num = (g_nodes[u].a00 + g_nodes[u].a01) % MODV
    let den = (g_nodes[u].a10 + g_nodes[u].a11) % MODV
    print(num, den)

let n: Int = read()
let q: Int = read()
let s: Str = read()
var root = -1
for i in 0..n: root = merge(root, make_node(s[i] == 'E'))
report(root)
for _ in 0..q:
    let ty: Str = read()
    if ty == "APPEND":
        let ch: Str = read()
        root = merge(root, make_node(ch[0] == 'E'))
    else:
        let l: Int = read()
        let r: Int = read()
        let p1 = split(root, l - 1)
        let p2 = split(p1.1, r - l + 1)
        let mid = p2.0
        if ty == "FLIP": g_nodes[mid].flip = not g_nodes[mid].flip
        else: g_nodes[mid].rev = not g_nodes[mid].rev
        pull(mid)
        root = merge(merge(p1.0, mid), p2.1)
    report(root)
