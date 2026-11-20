// IOI 2007 Pairs: n animals live on a line, a grid or a cube of side m
// (the first number of the input says which). Count the pairs that are at
// most d apart in Manhattan distance.
//
// Turning the board 45 degrees makes a Manhattan ball a box: on the grid,
// |dx| + |dy| <= d is |du| <= d and |dv| <= d for u = x + y, v = x - y.
//   line: sort, and count each animal against the window of animals within d
//         to its left.
//   grid: the same window over u, with the v of everyone in the window in a
//         Fenwick tree.
//   cube: m <= 75, so each layer z is a small turned grid with 2D prefix
//         sums. An animal asks every layer how many stand in its box there;
//         that counts each pair twice and each animal with itself once.
// original: corpus/submissions/3505539.cpp
use std.fenwick.{Fenwick}

let dims: Int = read()
let n: Int = read()
let d: Int = read()
let m: Int = read()

fn line() -> Int:
    var xs: List<Int> = []
    for _ in 0..n:
        let x: Int = read()
        xs.push(x)
    xs.sort()
    var (pairs, from) = (0, 0)
    for i in 0..n:
        while xs[i] - xs[from] > d: from += 1
        pairs += i - from
    pairs

fn grid() -> Int:
    domain V = 1 - m..=m - 1
    var at: List<(u: Int, v: Int)> = []
    for _ in 0..n:
        let x: Int = read()
        let y: Int = read()
        at.push((u: x + y, v: x - y))
    at.sort()
    var near = Fenwick<V, Int>()
    var (pairs, from) = (0, 0)
    for a in at:
        while a.u - at[from].u > d:
            near.add(V(at[from].v), -1)
            from += 1
        pairs += near.sum(V(max(a.v - d, 1 - m))..=V(min(a.v + d, m - 1)))
        near.add(V(a.v), 1)
    pairs

fn cube() -> Int:
    domain Z = 1..=m
    domain U = 1..=2 * m                    // x + y is 2..=2m; row 1 stays empty
    domain V = 0..=2 * m - 1                // x - y + m is 1..=2m-1; column 0 stays empty
    var at: List<(z: Int, u: Int, v: Int)> = []
    var seen: [Z, U, V]Int = 0
    for _ in 0..n:
        let x: Int = read()
        let y: Int = read()
        let z: Int = read()
        at.push((z: z, u: x + y, v: x - y + m))
        seen[Z(z), U(x + y), V(x - y + m)] += 1
    for z in Z:
        for u in U:
            for v in V:
                if u.label > 1 and v.label > 0:
                    seen[z, u, v] += seen[z, u.prev!, v] + seen[z, u, v.prev!] - seen[z, u.prev!, v.prev!]
    // animals of layer z with u in (u0..=u1) and v in (v0..=v1), all clamped to the board
    fn boxed(z: Z, u0: Int, u1: Int, v0: Int, v1: Int) -> Int:
        let (a, b) = (U(max(u0 - 1, 1)), U(min(u1, 2 * m)))
        let (c, e) = (V(max(v0 - 1, 0)), V(min(v1, 2 * m - 1)))
        seen[z, b, e] - seen[z, a, e] - seen[z, b, c] + seen[z, a, c]
    var twice = 0
    for p in at:
        for z in Z:
            let r = d - abs(z.label - p.z)
            if r >= 0: twice += boxed(z, p.u - r, p.u + r, p.v - r, p.v + r)
    (twice - n) / 2

print(if dims == 1: line() elif dims == 2: grid() else: cube())
