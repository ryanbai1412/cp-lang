// IOI 2007 Flood: n points and m axis-parallel walls between them; walls
// meet only at their end points, and a point has at most one wall in each
// direction. Water rises from outside and a wall falls when exactly one of
// its sides is wet; both sides then flood together. Print the walls left
// standing: those whose two sides flood at the same moment.
//
// The plane is cut into faces. Each wall has two sides; walking round a
// point, the left side of one wall and the right side of the next one are
// the same face, so a DSU over wall sides names the faces. The outside
// faces are found by sweeping points left to right: the first point of an
// untouched group has outside on its lower-left side. BFS from there counts
// the walls crossed, and a wall stands when both its faces are equally far.
//
// IDEAL SKETCH, not run: uses `Deque`, `argmin`-style helpers and the
// `gen` / `assume` hooks proposed in the backlog (Q1, Q2).
// original: corpus/submissions/5133897.cpp
use std.dsu.{Dsu}
use std.geometry.{Pt}

let n: 2..=1e5 = read()
domain P = 1..=n
let pts: [P]Pt = read()
let m: 1..=2e5 = read()
domain W = 1..=m
let walls: [W](u: P, v: P) = read(gen: gen_walls)
    assume all(pts[w.u].x == pts[w.v].x or pts[w.u].y == pts[w.v].y for w in walls)
    assume no_two_cross(pts, walls)

// Constructive generator: grow a random set of walls on a small lattice by
// only ever joining lattice neighbours. Neighbouring points cannot have a
// third point between them, and unit lattice walls cannot cross, so the
// `assume` lines above hold by construction and reject nothing.
fn gen_walls() -> [W](u: P, v: P):
    let at = Map((pts[p].x, pts[p].y) => p for p in P)
    let near = [(p, q) for p in P for d in [Pt(1, 0), Pt(0, 1)] if let q = at.get(pts[p] + d)]
    sample(near, m)

enum Dir: East, North, West, South  // counter-clockwise
domain Side = 0..2 * m              // wall w has sides 2w and 2w + 1
fn side(w: W, first: Bool) -> Side = Side.nth(2 * w.ord + if first: 0 else: 1)

fn heading(a: Pt, b: Pt) -> Dir:
    if a.y == b.y: (if b.x > a.x: East else: West) else: (if b.y > a.y: North else: South)

// out[p, d]: which side of the wall leaving p in direction d faces the
// next direction round p.
var out: [P, Dir]?Side = none
for w in W:
    let (u, v) = walls[w]
    let d = heading(pts[u], pts[v])
    out[u, d] = side(w, true)
    out[v, d.cyc(2)] = side(w, false)

var faces = Dsu<Side>.new()
for p in P:
    for d in Dir:
        let here = out[p, d] else: continue
        let next = first(out[p, d.cyc(k)] for k in 1..=4)!  // the next wall round p
        faces.unite(here, next.flip)                        // WISH: Side.flip = the other side

var across: [Side]List<Side> = []
for w in W:
    let (a, b) = (faces.find(side(w, true)), faces.find(side(w, false)))
    across[a].push(b)
    across[b].push(a)

var wet_at: [Side]?Int = none
var queue = Deque<Side>()
for p in sorted(P, by: p => (pts[p].x, pts[p].y)):
    // The leftmost, then lowest, point of a group: outside lies below its
    // east wall, or left of its north wall if it has no east wall.
    let outer = out[p, East] ?? out[p, North]?.flip else: continue
    let f = faces.find(outer)
    if wet_at[f] != none: continue
    wet_at[f] = 0
    queue.push(f)
    while let a = queue.pop_front():
        for b in across[a]:
            if wet_at[b] == none:
                wet_at[b] = wet_at[a]! + 1
                queue.push(b)

let standing = [
    w for w in W if wet_at[faces.find(side(w, true))] == wet_at[faces.find(side(w, false))]
]
print(standing.len)
for w in standing: print(w)
