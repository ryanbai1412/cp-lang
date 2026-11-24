// Source: NOI 2023 P4 (Trade), https://dmoj.ca/problem/noi23p4
// limits: 1500 ms, 512 MB
// Problem: 2^n - 1 cities (2 <= n <= 18) in a heap-shaped tree: city i >= 2 has a one-way road to
//   i/2 of length a_i, plus m <= 2^n directed roads u -> v with v in u's subtree (lengths <= 1e9).
//   Print the sum of dist(x, y) over all ordered pairs (0 if unreachable) mod 998244353.
// Solution: leaving x's subtree means first taking x's road up, so outside it
//   dist(x, y) = a_x + dist(parent, y). A DFS from the root keeps d[y] = dist(x, y) - dw(x),
//   dw being x's road sum to the root, in a RollbackArray: the parent's values already hold
//   every path via the parent, a Dijkstra seeded at x inside x's subtree improves the rest, and
//   the subtree's writes are undone on return. The sum and count of reached values ride down the
//   DFS as arguments. O((2^n + m) n log 2^n).
// original: corpus/submissions/5723063.cpp
use std.dsu.{RollbackArray}

type Z = Mod<998_244_353>

let (n, m): (2..=18, 0..=2 ** 18) = read()
domain City = 1..2 ** n
let up: [City.count - 1]1..=1e9 = read()  // city i >= 2's road to i / 2
let roads: [m](u: City, v: City, w: 1..=1e9) = read()

fn parent(c: City) -> City = City.nth((c.ord + 1) / 2 - 1)
fn road_up(c: City) -> Int = up[c.ord - 1]
let g = Graph.directed(roads.collect() + [(u: c, v: parent(c), w: road_up(c)) for c in City.drop(1)])

var d = RollbackArray<City, ?Int>(init: _ => None)
var total = Z(0)

fn walk(x: City, dw: Int, sum: Z, reached: Int):
    var (sum, reached) = (sum, reached)
    let mark = d.mark()
    var h = MinHeap<(Int, City)>()
    h.push((-dw, x))
    while let (du, u) = h.pop():
        if let cur = d.get(u):
            if cur <= du: continue
            sum -= Z(cur)
        else: reached += 1
        sum += Z(du)
        d.set(u, Some(du))
        for a in g.out(u):
            if a.to < x: continue  // x's road up, the only arc out of its subtree
            let nd = du + a.w
            if d.get(a.to).map(c => nd < c) ?? true: h.push((nd, a.to))
    total += sum + Z(dw) * Z(reached)
    for k in 2 * x.ord + 1..min(2 * x.ord + 3, City.count):
        let kid = City.nth(k)
        walk(kid, dw + road_up(kid), sum, reached)
    d.undo_to(mark)

walk(City.first!, 0, Z(0), 0)
print(total)
