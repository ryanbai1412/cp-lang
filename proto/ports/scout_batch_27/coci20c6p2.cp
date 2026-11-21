// COCI 2020/21 #6 Alias: a directed weighted graph whose n vertices are
// named by words, given as m lines `from to time`. For each of q queries
// `a b` print the shortest time from a to b, or "Roger" if b is unreachable.
// original: corpus/submissions/4188022.cpp
use std.seq.{get_or_insert_with}

let (n, m): (Int, Int) = read()
domain V = 0..n
var ids = Map<Str, V>()

fn id(name: Str) -> V:
    let next_id = ids.len
    ids.get_or_insert_with(name, () => V(next_id))

var out: [V]List<(to: V, time: Int)> = []
for _ in 0..m:
    let (a, b, time): (Str, Str, Int) = read()
    out[id(a)].push((to: id(b), time:))

let q: Int = read()
for _ in 0..q:
    let (a, b): (Str, Str) = read()
    let dist = dijkstra(v => out[v], [(id(a), 0)], e => e.time)
    if let d = dist[id(b)]: print(d)
    else: print("Roger")
