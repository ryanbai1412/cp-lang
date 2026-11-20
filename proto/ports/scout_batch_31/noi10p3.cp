// NOI 2010 P3, Altitude: an (n+1) x (n+1) lattice of junctions, every street
// segment carrying a flow in each of its two directions. Heights are 0 at the
// top-left and 1 at the bottom-right; choose the rest to minimise the total
// climbing effort. The answer is a minimum cut, found as a shortest walk in
// the dual: a walker threads between junctions from the left/bottom border
// to the top/right border, and each turn pays for the segments it sweeps
// across at the junction it passes.
// original: corpus/submissions/5672180.cpp
let n0: 1..=500 = read()
let n = n0 + 1                                  // junctions per side
domain X = 0..n
domain Y = 0..n
type Cell = (X, Y)
// Flows leaving each junction towards E, S, W, N, in the statement's order.
let east: [X, 0..n - 1]Int = read()
let south: [0..n - 1, Y]Int = read()
let west: [X, 1..n]Int = read()
let north: [1..n, Y]Int = read()

fn flow(p: Cell, d: Dir4) -> Int:               // FRICTION N1: four ragged edge arrays
    let (x, y) = (p.0.to_idx, p.1.to_idx)
    match d:
        Dir4.E: if y < n - 1: east[p.0, y] else: 0
        Dir4.S: if x < n - 1: south[x, p.1] else: 0
        Dir4.W: if y >= 1: west[p.0, y] else: 0
        Dir4.N: if x >= 1: north[x, p.1] else: 0

// A walker is at a junction having entered from `from`; leaving the lattice
// through the top or right border ends the walk.
enum Walk:
    At(p: Cell, from: Dir8)
    Out
let corner: Cell = (X.last, Y.last)

fn turns(w: Walk) -> Iter<(Walk, Int)>:
    if let Walk.At(p, from) = w and p != corner:
        var paid = 0
        var dir = from.ccw                      // FRICTION N2: sweep with a running cost
        for turn in 1..7:
            if turn > 1:
                match p.step(dir):
                    none: if dir in [Dir8.N, Dir8.E]: yield (Walk.Out, paid)
                    q: if q != corner: yield (Walk.At(q, dir.back), paid)
            if let axis = dir.as_dir4: paid += flow(p, axis)   // FRICTION N3: Dir8 -> ?Dir4
            dir = dir.ccw

let starts = chain((Walk.At((x, Y.first), Dir8.W) for x in X),
                   (Walk.At((X.last, y), Dir8.S) for y in Y))
print(dijkstra(turns, from: starts)[Walk.Out]!)
