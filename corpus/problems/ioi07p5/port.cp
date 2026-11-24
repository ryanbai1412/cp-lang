// Source: IOI 2007 Pairs, https://dmoj.ca/problem/ioi07p5
// limits: 1800 ms, 150 MB
// Problem: board type b in 1..=3 (a line, grid or cube with coordinates 1..=m), n <= 1e5
//   animals, d <= 1e8. Count the pairs of animals at Manhattan distance <= d; m <= 7.5e7,
//   75000, 75 for b = 1, 2, 3.
// Solution: turning the board 45 degrees makes a Manhattan ball a box: on the grid,
//   |dx| + |dy| <= d is |du| <= d and |dv| <= d for u = x + y, v = x - y.
//   line: sort, and count each animal against the window within d to its left, O(n log n).
//   grid: the same window over u, with the v's of the window in a Fenwick tree, O(n log m).
//   cube: each layer z is a small turned grid with 2D prefix sums; an animal asks every layer
//     how many stand in its box there, counting each pair twice and itself once, O(m^3 + n m).
//   Boxes are clipped to the turned board, so no window leaves it.
// original: corpus/submissions/3505539.cpp
// Bounds: dims: 1..=3; n: 1..=1e5; d: 1..=1e8; m: 1..=7.5e7; xs, at.x, at.y, at.z: 1..=m

let (dims, n, d, m): (1..=3, 1..=1e5, 1..=1e8, 1..=7.5e7) = read()

fn line() -> Int:
    let xs: [n]1..=m = read()
    let row = xs.sorted()
    var (pairs, from) = (0, 0)
    for (i, x) in row.items:
        while x - row[from] > d: from += 1
        pairs += i - from
    pairs

fn grid() -> Int:
    domain V = 1 - m..=m - 1
    let at: [n](x: 1..=m, y: 1..=m) = read()
    let turned = at.map(a => (u: a.x + a.y, v: V(a.x - a.y))).sorted()
    var near = Fenwick<V, Int>()
    var (pairs, from) = (0, 0)
    for a in turned:
        while a.u - turned[from].u > d:
            near.add(turned[from].v, -1)
            from += 1
        pairs += near.fold((a.v.prev(d) ?? V.first!)..=(a.v.next(d) ?? V.last!))
        near.add(a.v, 1)
    pairs

fn cube() -> Int:
    domain Z = 1..=m
    domain U = 2..=2 * m
    domain V = 1 - m..=m - 1
    let at: [n](x: 1..=m, y: 1..=m, z: 1..=m) = read()
    var seen: [Z, U, V]Int = 0
    for (x, y, z) in at: seen[Z(z), U(x + y), V(x - y)] += 1
    let layer = [Prefix<U, V, Int>([U, V] (u, v) => seen[z, u, v]) for z in Z]
    var twice = 0
    for (x, y, z) in at:
        for w in Z:
            let r = d - abs(w.label - z)
            if r >= 0:
                let (u, v) = (U(x + y), V(x - y))
                let rows = (u.prev(r) ?? U.first!)..=(u.next(r) ?? U.last!)
                twice += layer[w.ord].fold(rows, (v.prev(r) ?? V.first!)..=(v.next(r) ?? V.last!))
    (twice - n) / 2

print(if dims == 1: line() elif dims == 2: grid() else: cube())
