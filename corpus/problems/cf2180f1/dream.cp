// Codeforces 2180 F1: Control Car (Easy Version) (dream).
// Follow the car through an unbounded grid, choosing each corner only when the
// car first needs it. Two inherited corners say whether the floor and the right
// wall of the cell are shut (`Shut(floor, wall)`); the fresh bottom-right
// corner has 4 equally likely outcomes and decides the move:
//   - Fall down (only when `not floor`): the fresh corner becomes the next
//     cell's right wall (1 outcome shuts it, 2 leave it open -> `odds(wall, 2)`),
//     and the next cell's floor corner is fresh (`odds(floor, 3)`).
//   - Slide right: from `Shut(true, false)` (floored), 1 outcome shuts the next
//     floor and 2 leave it open; from `Shut(false, false)` (open), 1 outcome
//     slides right with the next floor open (`open + 2 * floored`). The next
//     wall corner is fresh (`odds(wall, 3)`).
//   - Stop in `(i, j)`: `Shut(false, true)` stops on 1 outcome (shuts floor),
//     `Shut(true, false)` stops on 1 outcome (shuts wall), and `Shut(true, true)`
//     stops on all 4 outcomes -> `walled + floored + 4 * boxed`.
// A stop in `(i, j)` has chosen `2(i + j) + 3` corners, so each choice carries
// weight `(1 / 4) ** (2(i + j) + 3)`; the car stops inside `n x m` iff
// `i < n` and `j < m`.
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

let q: [0..read(1..=10_000)](n: 1..=5000, m: 1..=5000) = read()
domain I = 0..q.map(x => x.n).max()!
domain J = 0..q.map(x => x.m).max()!
let inside = Prefix(stops<I, J>())
for (n, m) in q: print(Z(4) ** ((n + 1) * (m + 1)) * inside.prefix(I.cuts(n), J.cuts(m)))
