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
struct Shut(floor: Bool, wall: Bool)
fn odds(shut: Bool, open: Int) -> Int = if shut: 1 else: open

// stops[i, j]: the share of corner choices whose car stops in cell (i, j).
pub fn stops<domain I, domain J>() -> [I, J]Z:
    var ways: [J][Shut]Z = 0
    ways[J.first] = [s => odds(s.floor, 3) * odds(s.wall, 3)]
    let weight: [0..I.len + J.len]Z = [d => (Z(1) / 4) ** (2 * d + 3)]
    var out: [I, J]Z = 0
    for i in I:
        var side: [Shut]Z = 0
        for j in J:
            let here = ways[j] + side
            var fall_mass: Z = 0
            var slide_open: Z = 0
            var slide_shut: Z = 0
            var stop_mass: Z = 0
            for (s, w) in here.items:
                match s:
                    Shut(false, false) =>
                        fall_mass += w
                        slide_open += w
                    Shut(false, true) =>
                        fall_mass += w
                        stop_mass += w
                    Shut(true, false) =>
                        slide_open += 2 * w
                        slide_shut += w
                        stop_mass += w
                    Shut(true, true) =>
                        stop_mass += 4 * w
            ways[j] = [s => fall_mass * odds(s.wall, 2) * odds(s.floor, 3)]
            side = [s => (if s.floor: slide_shut else: slide_open) * odds(s.wall, 3)]
            out[i, j] = stop_mass * weight[i + j]
    out

let q: [0..read(1..=10_000)](n: 1..=5000, m: 1..=5000) = read()
domain I = 0..max(q.n)
domain J = 0..max(q.m)
let inside = stops<I, J>().prefix_sums()
for (n, m) in q: print(Z(4) ** ((n + 1) * (m + 1)) * inside[..n, ..m])
