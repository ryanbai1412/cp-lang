// Codeforces 2180 F2: Control Car (Hard Version) (dream).
// Follow the car through an unbounded grid, choosing each corner only when the
// car first needs it. Two inherited corners say whether the floor and the right
// wall of the cell are shut; the fresh bottom-right corner decides the move.
// A fall hands that corner down as the next top-right one, a slide hands it
// right as the next bottom-left one. A stop in (i, j) has chosen 2(i + j) + 3
// corners; the car stops inside the n x m grid iff it stops at i < n, j < m.
// For fixed n the count is a linear recurrence in m: tabulate and extend it.
use std.spec.{guess_at}

type Z = Mod<1_000_000_007>

// A cell's state is whether its floor and its wall are shut; the four masses are kept by name
// until a struct can key an array (Core: struct-as-domain).
type Ways = (open: Z, walled: Z, shut: Z, boxed: Z)
fn state(open: Z, shut: Z, floor: Int, wall: Int) -> Ways =
    (open * floor * wall, open * floor, shut * wall, shut)

fn stops<domain I, domain J>() -> [I, J]Z:
    var ways: [J]Ways = (0, 0, 0, 0)
    ways[J.first!] = state(1, 1, floor: 3, wall: 3)
    let weight = [(Z(1) / 4) ** (2 * d + 3) for d in 0..I.len + J.len]
    var out: [I, J]Z = 0
    for i in I:
        var side: Ways = (0, 0, 0, 0)
        for j in J:
            let w = ways[j]
            let (open, walled, shut) = (w.open + side.open, w.walled + side.walled, w.shut + side.shut)
            out[i, j] = (walled + shut + 4 * (w.boxed + side.boxed)) * weight[i.ord + j.ord]
            let falls = open + walled
            ways[j] = state(falls, falls, floor: 3, wall: 2)
            side = state(open + 2 * shut, shut, floor: 1, wall: 3)
    out

domain I = 0..50
domain J = 0..600
let inside = Prefix(stops<I, J>())

@!multitest
let (n, m): (1..=I.len, 1..=1e15) = read()
print(guess_at([Z(4) ** ((n + 1) * (Int(k) + 1)) * inside.prefix(I.cuts(n), k) for k in J.cuts], m)!)
