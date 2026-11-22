// A simple graph, a connected one, a DAG and a digraph read by their stdlib types.
use std.graph.{ConnectedGraph, Dag, Digraph, SimpleGraph}

let (n, m): (1..=10, 0..=45) = read()
domain V = 1..=n
domain E = 0..m
let a: SimpleGraph<V, E> = read()
let b: ConnectedGraph<V, E> = read()
let c: Dag<V, E> = read()
let d: Digraph<V, E> = read()
print(a.len, b.len, c.len, d.len)
for (u, v) in c: print(u.label, v.label)
for (u, v) in d: print(u.label, v.label)
