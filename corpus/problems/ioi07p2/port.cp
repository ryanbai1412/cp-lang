// Source: IOI 2007 Flood, https://dmoj.ca/problem/ioi07p2
// Problem: n <= 1e5 points (coordinates <= 1e6) and w <= 2n axis-parallel walls joining pairs of
//   them, touching only at end points. At time 0 water floods the outside; every hour each wall
//   with water on exactly one side breaks and the water spreads. Print how many walls are left
//   standing, then their indices.
// Solution: the walls cut the plane into faces, and a wall stands iff its two faces flood in the
//   same hour. A DSU over wall sides names the faces: round each point, a wall's side joins the
//   far side of the next wall counter-clockwise. Sweep points by (x, y): the first point of a
//   still-dry group has the outside next to its north (else east) wall; BFS over faces from
//   there, one hour per wall crossed. O((n + w) log n) for the sort and DSU.
// limits: 1000 ms, 32 MB
// original: corpus/submissions/5133897.cpp
// Bounds: n: 2..=1e5; m: 1..=2e5; walls.u, walls.v: 1..=n

let n: 2..=1e5 = read()
domain P = 1..=n
let pts: [P]Pt = read()
let m: 1..=2e5 = read()
domain W = 1..=m
let walls: [W](u: P, v: P) = read()

enum Dir = East | North | West | South  // counter-clockwise
domain Side = 0..2 * m              // wall w has sides 2w and 2w + 1
fn side(w: W, first: Bool) -> Side = Side.nth(2 * w.ord + if first: 0 else: 1)
fn flip(s: Side) -> Side = Side.nth(s.ord ^ 1)  // the other side of the same wall

let dirs = [d for d in Dir]
fn turn(d: Dir, k: Int) -> Dir = dirs[((0..4).find(i => dirs[i] == d)! + k) % 4]  // k quarter turns counter-clockwise

fn heading(a: Pt, b: Pt) -> Dir:
    if a.y == b.y: (if b.x > a.x: East else: West) else: (if b.y > a.y: North else: South)

// out[p, d]: which side of the wall leaving p in direction d faces the
// next direction round p.
var out: [P, Dir]?Side = None
for w in W:
    let (u, v) = walls[w]
    let d = heading(pts[u], pts[v])
    out[u, d] = side(w, true)
    out[v, turn(d, 2)] = side(w, false)

var faces = Dsu<Side>.new()
for p in P:
    for d in Dir:
        let here = out[p, d] else: continue
        var next = here  // the next wall round p
        for k in 1..=4:
            if let s = out[p, turn(d, k)]:
                next = s
                break
        faces.unite(here, flip(next))

var across: [Side]List<Side> = []
for w in W:
    let (a, b) = (faces.find(side(w, true)), faces.find(side(w, false)))
    across[a].push(b)
    across[b].push(a)

var wet_at: [Side]?Int = None
var queue = Deque<Side>()
for p in sorted(P, by: p => (pts[p].x, pts[p].y)):
    // The leftmost, then lowest, point of a group has no west or south wall:
    // outside is the face turning counter-clockwise from its north wall, or
    // from its east wall if it has no north wall.
    var outer = out[p, North]
    if outer == None: outer = out[p, East]
    let start = outer else: continue
    let f = faces.find(start)
    if wet_at[f] != None: continue
    wet_at[f] = 0
    queue.push(f)
    while let a = queue.pop_front():
        for b in across[a]:
            if wet_at[b] == None:
                wet_at[b] = wet_at[a]! + 1
                queue.push(b)

let standing = [
    w for w in W if wet_at[faces.find(side(w, true))] == wet_at[faces.find(side(w, false))]
]
print(standing.len)
for w in standing: print(w)
