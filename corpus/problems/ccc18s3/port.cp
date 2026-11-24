// Source: CCC 2018 S3 (RoboThieves), https://dmoj.ca/problem/ccc18s3
// Problem: an n x m grid (4 <= n, m <= 100) walled on the border: '.', 'W', cameras 'C',
//   conveyors L R U D and the start S. Cameras see along rows and columns up to a wall, over
//   conveyors; the robot may not stand on a seen cell or a camera. Conveyors move it for free.
//   For each '.' in row-major order print the fewest steps to reach it, or -1.
// Solution: walk each camera's four rays and mark every seen non-conveyor cell. Then a 0-1
//   BFS by levels: a conveyor moves to its next cell at cost 0, any other cell steps to its
//   four neighbours at cost 1, skipping walls and seen cells. O(nm) plus the rays.
// original: corpus/submissions/4329580.cpp
// Bounds: n, m: 4..=100
use std.graph.{bfs01}

let n: 4..=100 = read()
let m: 4..=100 = read()
domain Row = 1..=n
domain Col = 1..=m
let g: [Row, Col]Char = read()
let cells = [(r, c) for r in Row for c in Col]

fn belt(p: (Row, Col)) -> ?Dir4 = Dir4.from("URDL", g[p])

// A camera's line of sight stops at a wall and passes over conveyors.
var seen: [Row, Col]Bool = false
for cam in cells if g[cam] == 'C':
    for d in Dir4.all():
        for q in cam.ray(d).take_while(q => g[q] != 'W') if belt(q) == None:
            seen[q] = true

// Riding a conveyor is free; a step off any other cell costs one.
fn moves(p: (Row, Col)) -> List<(to: (Row, Col), w: Int)>:
    let next = if let d = belt(p): [(p.step(d)!, 0)] else: [(q, 1) for q in p.adj4]
    [(to: q, w:) for (q, w) in next if g[q] != 'W' and not seen[q]]

let start = cells.find(p => g[p] == 'S')!
let dist = bfs01(moves, from: if seen[start]: [] else: [start])
for p in cells if g[p] == '.': print(dist[p] ?? -1)
