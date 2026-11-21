// Codeforces 2180 F1: Control Car (Hard Version) (dream).
// Follow the car through an unbounded grid, choosing each corner only when the
// car first needs it. Two inherited corners say whether the floor and the right
// wall of the cell are shut; the fresh bottom-right corner decides the move.
// A fall hands that corner down as the next top-right one, a slide hands it
// right as the next bottom-left one. A stop in (i, j) has chosen 2(i + j) + 3
// corners; the car stops inside the n x m grid iff it stops at i < n, j < m.
// For fixed n the count is a linear recurrence in m: tabulate and extend it.
type Z = Mod<1e9 + 7>
record Shut(floor: Bool, wall: Bool)
fn odds(shut: Bool, open: Int) -> Int = if shut: 1 else: open

domain I = 0..50
domain J = 0..600

var ways: [J][Shut]Z = 0
ways[J.first] = [s => odds(s.floor, 3) * odds(s.wall, 3)]
let weight: [0..I.len + J.len]Z = [d => (1 / 4) ** (2 * d + 3)]
var stops: [I, J]Z = 0
for i in I:
    var side: [Shut]Z = 0
    for j in J:
        let here = ways[j] + side
        let falls = here[(false, false)] + here[(false, true)]
        let slides = here[(false, false)] + 2 * here[(true, false)]
        ways[j] = [s => falls * odds(s.wall, 2) * odds(s.floor, 3)]
        side = [s => (if s.floor: here[(true, false)] else: slides) * odds(s.wall, 3)]
        stops[i, j] = (here[(false, true)] + here[(true, false)] + 4 * here[(true, true)]) * weight[i + j]

let inside = stops.prefix_sums()
--- multitest ---
let (n, m): (1..=50, 1..=1e15) = read()
print(guess_at([j => 4 ** ((n + 1) * (j + 2)) * inside[..n, ..=j]], at: m - 1)!)
