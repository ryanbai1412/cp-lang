// Source: IOI 2022 Day 2 P1, Digital Circuit, https://dmoj.ca/problem/ioi22p4
// Problem: a tree of N threshold gates and M source gates (N, M <= 1e5, P[i] < i); a gate with
//   c inputs takes a parameter p in 1..=c and is 1 iff at least p inputs are 1. Q <= 1e5 times,
//   toggle sources L..=R and count parameter assignments making gate 0 output 1, mod 1000002022.
// Solution: by linearity each gate passes a count through once, so the answer is the sum over
//   lit sources of the product of all[] (assignments in a subtree) over the subtrees hanging
//   off the source's root path. Compute all[] bottom-up and worth[] top-down with prefix and
//   suffix products (the modulus is not prime). A lazy segment tree over sources keeps
//   (lit worth, total worth) under range flips. O(N + M + Q log M).
// original: corpus/submissions/4783516.cpp (grader functions init / count_ways)
// stdin: n m q, the n + m parents (-1 for gate 0), the m source states, then
// q lines l r. stdout: the count after each toggle.
const MOD = 1000002022
let n: 1..=100000 = read()
let m: 1..=100000 = read()
let q: 0..=100000 = read()
let parent: [0..n + m]Int = read()
let lit: [0..m]Int = read()

var kids: List<List<Int>> = [[] for _ in 0..n]
for v in 1..n + m: kids[parent[v]].push(v)

// all[u]: parameter assignments inside u's subtree (a gate with c inputs has c choices)
var all: List<Int> = List.filled(n + m, 1)
for u in (0..n).rev:
    all[u] = kids[u].len
    for v in kids[u]: all[u] = all[u] * all[v] % MOD

// A lit source is worth the assignments of everything off its path to the
// root: by linearity each gate on the path passes the count through once.
// MOD is not prime, so "all but one child" is a prefix times a suffix product.
var worth: List<Int> = List.filled(n + m, 1)
for u in 0..n:
    let c = kids[u].len
    var after: List<Int> = [1 for _ in 0..=c]
    for i in (0..c).rev: after[i] = after[i + 1] * all[kids[u][i]] % MOD
    var before = worth[u]
    for i in 0..c:
        worth[kids[u][i]] = before * after[i + 1] % MOD
        before = before * all[kids[u][i]] % MOD

// Lazy tree over the sources: (worth of lit sources, worth of all sources), range flip.
var on: List<Int> = []
var total: List<Int> = []
var flip: List<Bool> = []
for _ in 0..4 * m:
    on.push(0)
    total.push(0)
    flip.push(false)

fn build(o: Int, l: Int, r: Int):
    if l == r:
        total[o] = worth[n + l]
        on[o] = if lit[l] == 1: total[o] else: 0
        return
    let mid = (l + r) / 2
    build(2 * o, l, mid)
    build(2 * o + 1, mid + 1, r)
    total[o] = (total[2 * o] + total[2 * o + 1]) % MOD
    on[o] = (on[2 * o] + on[2 * o + 1]) % MOD

fn toggle_node(o: Int):
    on[o] = (total[o] - on[o]) % MOD
    flip[o] = not flip[o]

fn toggle(o: Int, l: Int, r: Int, ql: Int, qr: Int):
    if ql <= l and r <= qr:
        toggle_node(o)
        return
    if flip[o]:
        toggle_node(2 * o)
        toggle_node(2 * o + 1)
        flip[o] = false
    let mid = (l + r) / 2
    if ql <= mid: toggle(2 * o, l, mid, ql, qr)
    if qr > mid: toggle(2 * o + 1, mid + 1, r, ql, qr)
    on[o] = (on[2 * o] + on[2 * o + 1]) % MOD

build(1, 0, m - 1)
for _ in 0..q:
    let l: Int = read()
    let r: Int = read()
    toggle(1, 0, m - 1, l - n, r - n)
    print(on[1])
