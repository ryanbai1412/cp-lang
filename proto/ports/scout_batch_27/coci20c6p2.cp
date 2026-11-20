// COCI 2020/21 #6 Alias: a directed weighted graph whose n vertices are
// named by words, given as m lines `from to time`. For each of q queries
// `a b` print the shortest time from a to b, or "Roger" if b is unreachable.
// original: corpus/submissions/4188022.cpp
use std.graph.{dijkstra}

let n: Int = read()
let m: Int = read()
domain V = 0..n
var ids = Map<Str, V>()
fn id(name: Str) -> V:
    if name not in ids: ids[name] = V(ids.len)
    ids[name]

var out: [V]List<(to: V, time: Int)> = []
for _ in 0..m:
    let a: Str = read()
    let b: Str = read()
    let time: Int = read()
    out[id(a)].push((to: id(b), time: time))

let q: Int = read()
for _ in 0..q:
    let a: Str = read()
    let b: Str = read()
    let dist = dijkstra(v => out[v], [(id(a), 0)], e => e.time)
    if let d = dist[id(b)]: print(d)
    else: print("Roger")
