// Codeforces 2180 F1: Control Car (Easy Version) (dream).
// Follow the car through an unbounded grid, choosing each corner only when the
// car first needs it. Two inherited corners say whether the floor and the right
// wall of the cell are shut; the fresh bottom-right corner decides the move.
// A fall hands that corner down as the next top-right one, a slide hands it
// right as the next bottom-left one. A stop in (i, j) has chosen 2(i + j) + 3
// corners; the car stops inside the n x m grid iff it stops at i < n, j < m.
type Z = Mod<1e9 + 7>
struct Shut(floor: Bool, wall: Bool)
fn odds(shut: Bool, open: Int) -> Int = if shut: 1 else: open

// stops[i, j]: the share of corner choices whose car stops in cell (i, j).
pub fn stops<domain I, domain J>() -> [I, J]Z:
    var ways: [J][Shut]Z = 0
    ways[J.first] = [s => odds(s.floor, 3) * odds(s.wall, 3)]
    let weight: [0..I.len + J.len]Z = [d => (1 / 4) ** (2 * d + 3)]
    var stops: [I, J]Z = 0
    for i in I:
        var side: [Shut]Z = 0
        for j in J:
            let here = ways[j] + side
            let (open, walled) = (here[Shut(floor: false, wall: false)], here[Shut(floor: false, wall: true)])
            let (floored, boxed) = (here[Shut(floor: true, wall: false)], here[Shut(floor: true, wall: true)])
            ways[j] = [s => (open + walled) * odds(s.wall, 2) * odds(s.floor, 3)]
            side = [s => (if s.floor: floored else: open + 2 * floored) * odds(s.wall, 3)]
            stops[i, j] = (walled + floored + 4 * boxed) * weight[i + j]
    stops

let q: [0..read(1..=1e4)](n: 1..=5000, m: 1..=5000) = read()
domain I = 0..max(q.n)
domain J = 0..max(q.m)
let inside = stops<I, J>().prefix_sums()
for (n, m) in q: print(4 ** ((n + 1) * (m + 1)) * inside[..n, ..m])
