// Source: COCI 2020/2021 Contest 6 (Alias), https://dmoj.ca/problem/coci20c6p2
// Problem: n <= 1000 words and m <= 1000 directed associations `x y t` (t <= 1e9): hearing x
//   makes you think of y after t ms. For each of q <= 1000 queries `a b`, print the shortest
//   time from a to b, or "Roger" if b is never reached.
// Solution: give each word an id on first sight (a hash map) and build the weighted digraph;
//   run Dijkstra from a for each query: O(q (n + m) log n).
// original: corpus/submissions/4188022.cpp
use std.marks.{Interner}

let (n, m): (Int, Int) = read()
domain V = 0..n
var ids = Interner<Str>()

fn id(name: Str) -> V:
    V(ids.id(name))

var out: [V]List<(to: V, w: Int)> = []
for _ in 0..m:
    let (a, b, time): (Str, Str, Int) = read()
    out[id(a)].push((to: id(b), w: time))

let q: Int = read()
for _ in 0..q:
    let (a, b): (Str, Str) = read()
    let dist = dijkstra(out, from: [(id(a), 0)])
    if let d = dist[id(b)]: print(d)
    else: print("Roger")
