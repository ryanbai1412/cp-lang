// NOIP 2010 Imprisonment: each feud between two inmates has an anger. Split
// the inmates over two prisons so that the angriest feud inside one prison
// is as mild as possible; print that anger, 0 if no feud stays inside.
// original: corpus/submissions/5205348.cpp
use std.graph.{Graph, two_colour}                                   // WISH F1, K8

let n: 1..=2e4 = read()
let m: 1..=1e5 = read()
domain Inmate = 1..=n
let feuds: [m](a: Inmate, b: Inmate, anger: 1..=1e9) = read()

// Feuds angrier than x must be split: possible exactly when they 2-colour.
fn peaceful(x: Int) -> Bool:
    let foes = Graph.undirected(f for f in feuds if f.anger > x)    // WISH K8: a graph of some of the edges
    two_colour(foes.adj) != none
print((0..=1e9).first_where(peaceful)!)                             // WISH K6
