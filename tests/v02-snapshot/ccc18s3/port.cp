// Source: CCC 2018 S3 (RoboThieves), https://dmoj.ca/problem/ccc18s3
// Problem: an n x m grid (4 <= n, m <= 100) walled on the border: '.', 'W', cameras 'C',
//   conveyors L R U D and the start S. Cameras see along rows and columns up to a wall, over
//   conveyors; the robot may not stand on a seen cell or a camera. Conveyors move it for free.
//   For each '.' in row-major order print the fewest steps to reach it, or -1.
// Solution: walk each camera's four rays and mark every seen non-conveyor cell. Then a 0-1
//   BFS by levels: a conveyor moves to its next cell at cost 0, any other cell steps to its
//   four neighbours at cost 1, skipping walls and seen cells. O(nm) plus the rays.
// original: corpus/submissions/4329580.cpp
let n: 4..=100 = read()
let m: 4..=100 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let g: [Row, Col]Char = read()

const DIRS = [(-1, 0), (1, 0), (0, -1), (0, 1)]
fn step(p: Cell, d: (Int, Int)) -> ?Cell:  // WISH L1: p.step(d) over Dir4
    if let r = Row.at(p.0 + d.0) and let c = Col.at(p.1 + d.1):
        return (r, c)
    return None

fn belt(ch: Char) -> ?(Int, Int):  // WISH L2: Dir4.from("UDLR", ch)
    if ch == 'U': return DIRS[0]
    if ch == 'D': return DIRS[1]
    if ch == 'L': return DIRS[2]
    if ch == 'R': return DIRS[3]
    return None

fn cells() -> List<Cell>: [(r, c) for r in Row for c in Col]

// A camera's line of sight stops at a wall and passes over conveyors.
var seen: [Row, Col]Bool = false
for cam in cells():  // WISH L5: `for cam in Cell` over a product domain
    if g[cam] == 'C':
        for d in DIRS:
            var at: ?Cell = cam  // WISH L3: for q in cam.ray(d).take_while(q => g[q] != 'W')
            while let q = at:
                if g[q] == 'W': break
                if belt(g[q]) == None: seen[q] = true
                at = step(q, d)

// Zero-one search: riding a conveyor is free, a step off any other cell costs one.
var dist: [Row, Col]?Int = None
var done: [Row, Col]Bool = false
var near: List<Cell> = []  // WISH L4: a Deque; WISH L7: shortest(moves, from: start) with weights 0..=1
var far: List<Cell> = []
for start in cells():  // WISH L6: let start = g.find('S')!
    if g[start] == 'S' and not seen[start]:
        dist[start] = 0
        near.push(start)
while near.len > 0 or far.len > 0:
    if near.len == 0:
        near = far
        far = []
    let p = near.pop()!
    if not done[p]:
        done[p] = true
        let here = dist[p]!
        let ride = belt(g[p])
        let cost = if ride == None: 1 else: 0
        let moves = if ride == None: DIRS else: [ride!]
        for d in moves:
            if let q = step(
                p,
                d,
            ) and g[q] != 'W' and not seen[q] and (dist[q] ?? (n * m)) > here + cost:
                dist[q] = here + cost
                if cost == 0: near.push(q) else: far.push(q)

for p in cells():
    if g[p] == '.': print(dist[p] ?? -1)
