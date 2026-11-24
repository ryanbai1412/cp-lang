// Source: BOI 2020 P3 (Joker), https://dmoj.ca/problem/btoi20p3
// limits: 1000 ms, 256 MB
// Problem: a graph with n, m, q <= 2e5; each query l r removes edges l..=r. Print YES if the
//   remaining edges contain an odd cycle, else NO.
// Solution: limit[l] = the least R such that edges [1, l) and (R, m] are bipartite; it is
//   non-decreasing in l, and a query says YES iff r < limit[l]. Divide and conquer on l with
//   limit's range narrowing, over a parity DSU with union by size and rollback (no path
//   compression): each level adds the edges its halves share, then undoes them.
//   O(m log m log n).
// original: corpus/submissions/4656866.cpp

let (n, m, q): (Int, Int, Int) = read()
domain V = 1..=n
domain E = 1..=m
domain Q = 0..q
let edges: [E](u: V, v: V) = read()
let queries: [Q](l: E, r: E) = read()

var up: [V]V = [V] v => v
var sz: [V]Int = [V] _ => 1
var par: [V]Bool = [V] _ => false
var hist_child: List<V> = []
var hist_parent: List<V> = []
var hist_sz: List<Int> = []
var odd_cycle = 0

fn find_root(v: V) -> (root: V, flip: Bool):
    var x = v
    var f = false
    while up[x] != x:
        f = f != par[x]
        x = up[x]
    (root: x, flip: f)

fn add_edge(u: V, v: V):
    let ru = find_root(u)
    let rv = find_root(v)
    if ru.root == rv.root:
        if ru.flip == rv.flip: odd_cycle += 1
        return
    var a = ru.root
    var b = rv.root
    if sz[a] < sz[b]:
        swap(mut a, mut b)
    hist_child.push(b)
    hist_parent.push(a)
    hist_sz.push(sz[a])
    up[b] = a
    par[b] = ru.flip == rv.flip
    sz[a] += sz[b]

fn rollback(h: Int, prev_odd: Int):
    odd_cycle = prev_odd
    while hist_child.len > h:
        let c = hist_child.pop()!
        let p = hist_parent.pop()!
        let os = hist_sz.pop()!
        up[c] = c
        par[c] = false
        sz[p] = os

var limit: [E]Int = [E] _ => m + 1

fn dnc(l_lo: Int, l_hi: Int, r_lo: Int, r_hi: Int):
    if l_lo > l_hi: return
    let mid = (l_lo + l_hi) / 2
    let h0 = hist_child.len
    let odd0 = odd_cycle
    for i in l_lo..mid:
        let e = edges[E(i)]
        add_edge(e.u, e.v)

    var opt = m + 1
    if odd_cycle == 0:
        opt = r_hi
        var cur = r_hi
        let stop_r = max(r_lo, mid)
        while cur >= stop_r:
            let e = edges[E(cur)]
            add_edge(e.u, e.v)
            if odd_cycle > 0: break
            opt = cur - 1
            cur -= 1
    limit[E(mid)] = opt
    rollback(h0, odd0)

    let left_r_hi = min(opt, r_hi)
    if l_lo <= mid - 1:
        let hl = hist_child.len
        let oddl = odd_cycle
        for i in (left_r_hi + 1)..=r_hi:
            let e = edges[E(i)]
            add_edge(e.u, e.v)
        dnc(l_lo, mid - 1, r_lo, left_r_hi)
        rollback(hl, oddl)

    let right_r_lo = max(r_lo, min(opt, m))
    if mid + 1 <= l_hi:
        let hr = hist_child.len
        let oddr = odd_cycle
        for i in l_lo..=mid:
            let e = edges[E(i)]
            add_edge(e.u, e.v)
        dnc(mid + 1, l_hi, right_r_lo, r_hi)
        rollback(hr, oddr)

dnc(1, m, 1, m)
for qi in Q:
    let ql = queries[qi].l
    let qr = queries[qi].r.ord + 1
    print(if qr < limit[ql]: "YES" else: "NO")
