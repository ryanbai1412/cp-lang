// Runnable port (proto/cp.py lib-run) of statement-first btoi17p2.cp
// original: corpus/submissions/4456186.cpp
use std.segtree.{SegTree}
use std.algebra.{Monoid}

let (k, n, m, o): (Int, Int, Int, Int) = read()
domain E = 0..m
domain Q = 0..o
domain Slot = 0..k
let edges: [E](a: Int, b: Int, t: Int) = read()
let queries: [Q](a: Int, b: Int) = read()

let layers = (n + k - 1) / k
let num_gaps = if layers > 1: layers - 1 else: 1
domain Gap = 0..num_gaps
type Mat = [Slot, Slot]?Int

fn mat_id() -> Mat:
    [Slot, Slot] (i, j) => if i == j: 0 else: none

fn mat_empty() -> Mat:
    [Slot, Slot] (i, j) => none

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

let st = SegTree<Gap, Mat>.from(min_plus, step)
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
