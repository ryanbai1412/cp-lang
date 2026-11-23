// Source: NOI 2002 P1 (Galactic Heroes), https://dmoj.ca/problem/noi02p1
// Problem: 30000 ships, each its own column. T <= 5e5 lines: "M i j" moves i's whole column
//   behind j's (they differ); "C i j" prints how many ships stand between i and j, or -1 if
//   they are in different columns.
// Solution: weighted union-find: each ship keeps its distance to its parent and each root its
//   column's size; a merge hangs i's root under j's at distance size(j). Path compression adds
//   up distances, so a query is |d_i - d_j| - 1. O((T + n) alpha(n)).
// cp: 0.1 8ef64b9+dirty
domain Ship = 1..=30000

@tags("M", "C")
enum Op:
    Merge(a, b: Ship)
    Ask(a, b: Ship)

let q: 1..=5e5 = read()
domain Query = 0..q

// few distinct ships, so columns actually meet
fn random_op() -> Op:
    let (a, b) = (Ship(rand(1, 5)), Ship(rand(1, 5)))
    if rand(0, 1) == 0: Merge(a, b) else: Ask(a, b)

// ops are handled as they are read, never stored
fn ops() -> yields Op:
    for _ in Query:
        let op: Op = read(gen: random_op)
        yield op

var toward: [Ship]?Ship = None  // a ship nearer the front of the same column
var gap: [Ship]Int = 0          // distance from u to toward[u]
var size: [Ship]Int = 1         // column size, valid at front ships

// (front ship of u's column, ships in front of u)
fn locate(u: Ship) -> (Ship, Int):
    if let p = toward[u]:
        let (f, d) = locate(p)
        gap[u] += d
        toward[u] = f
        return (f, gap[u])
    (u, 0)

for op in ops():
    match op:
        Merge(a, b) =>
            let (fa, da) = locate(a)
            let (fb, db) = locate(b)
            if fa != fb:
                toward[fa] = fb
                gap[fa] = size[fb]
                size[fb] += size[fa]
        Ask(a, b) =>
            let (fa, da) = locate(a)
            let (fb, db) = locate(b)
            print(if fa == fb: abs(da - db) - 1 else: -1)
