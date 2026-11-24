// Source: BOI 2017 P2 (Toll), https://dmoj.ca/problem/btoi17p2
// limits: 1000 ms, 1024 MB
// Problem: n <= 5e4 places, m one-way roads with tolls, each from a to b with
//   b / k = a / k + 1 (k <= 5). For o <= 1e4 orders (a, b) print the cheapest toll from a
//   to b, or -1.
// Solution: roads only join consecutive blocks of k places, so each gap between blocks is a
//   k x k min-plus matrix and a route is their product. A segment tree over the gaps answers
//   each order by folding gaps a / k ..= b / k - 1 and reading entry (a % k, b % k).
//   O(k^3 (n / k + o log n)).
// original: corpus/submissions/4456186.cpp

let (k, n, m, o): (Int, Int, Int, Int) = read()
domain E = 0..m
domain Q = 0..o
domain Slot = 0..k
let edges: [E](a: Int, b: Int, t: Int) = read()
let queries: [Q](a: Int, b: Int) = read()

let layers = ceil_div(n, k)
let num_gaps = if layers > 1: layers - 1 else: 1
domain Gap = 0..num_gaps
type Mat = [Slot, Slot]?Int

fn mat_id() -> Mat:
    [Slot, Slot] (i, j) => if i == j: 0 else: None

fn mat_empty() -> Mat:
    [Slot, Slot] (i, j) => None

fn mat_mul(x: Mat, y: Mat) -> Mat:
    var out: Mat = mat_empty()
    for i in Slot:
        for mid in Slot:
            if let d1 = x[i, mid]:
                for j in Slot:
                    if let d2 = y[mid, j]:
                        if let cur = out[i, j]:
                            if d1 + d2 < cur: out[i, j] = d1 + d2
                        else:
                            out[i, j] = d1 + d2
    out

let min_plus = Monoid<Mat>(id: mat_id(), op: (x, y) => mat_mul(x, y))

var step: [Gap]Mat = [Gap] _ => mat_empty()
for e in E:
    let (a, b, t) = (edges[e].a, edges[e].b, edges[e].t)
    let g = Gap(a / k)
    let (sa, sb) = (Slot(a % k), Slot(b % k))
    if let cur = step[g][sa, sb]:
        if t < cur: step[g][sa, sb] = t
    else:
        step[g][sa, sb] = t

let st = SegTree<Gap, Mat>(min_plus, step)
for q in Q:
    let (a, b) = (queries[q].a, queries[q].b)
    let (ga, sa) = (a / k, Slot(a % k))
    let (gb, sb) = (b / k, Slot(b % k))
    if ga == gb:
        print(if a == b: 0 else: -1)
    elif ga > gb:
        print(-1)
    else:
        let prod = st.fold(Gap(ga)..=Gap(gb - 1))
        print(prod[sa, sb] ?? -1)
