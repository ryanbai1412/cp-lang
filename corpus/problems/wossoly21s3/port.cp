// Source: WOSS Dual Olympiad 2021 S3 (Late for Class!), https://dmoj.ca/problem/wossoly21s3
// Problem: an R x C grid (R, C <= 200) of '.', '#' walls, marks 0-4 and letter portals: stepping
//   onto capital X teleports you for free to its small x, and small letters can't be entered.
//   Walk 0 -> 1 -> 2 -> 3 -> 4 in order (4-neighbour steps, 1 s each); print the fewest seconds,
//   or -1 if some class can't be reached.
// Solution: the legs are independent, so add four shortest paths. Each is a BFS over cells where
//   a step onto q lands at land(q): the small letter for a capital, nowhere for '#' or a small
//   letter, q itself otherwise. O(RC) per leg; an unreachable leg makes the total -1.
// original: corpus/submissions/4224399.cpp
let n: 1..=200 = read()
let m: 1..=200 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let school: [Row, Col]Char = read()

const DIRS = [(-1, 0), (1, 0), (0, -1), (0, 1)]
fn step(p: Cell, d: (Int, Int)) -> ?Cell:  // WISH L1: p.step(d) over Dir4
    if let r = Row.at(p.0 + d.0) and let c = Col.at(p.1 + d.1):
        return (r, c)
    return None

var place = Map<Char, Cell>()  // WISH L6: school.find('a')
for r in Row:
    for c in Col: place[school[r, c]] = (r, c)

// Where a step onto q really puts you.
fn land(q: Cell) -> ?Cell:
    let ch = school[q]
    if ch == '#' or ('a' <= ch and ch <= 'z'): return None  // WISH L9: ch.is_lower, ch.lower
    if 'A' <= ch and ch <= 'Z': return place[ch + 32]
    return q

fn walk(from: Cell, to: Cell) -> ?Int:
    var dist: [Row, Col]?Int = None  // WISH L7: bfs(from, p => (land(q) for q in p.around()))[to]
    var todo: List<Cell> = [from]
    var next = 0
    dist[from] = 0
    while next < todo.len:
        let p = todo[next]
        next += 1
        for d in DIRS:
            if let q = step(p, d):
                if let at = land(q):
                    if dist[at] == None:
                        dist[at] = dist[p]! + 1
                        todo.push(at)
    return dist[to]

fn total() -> ?Int:
    var steps = 0
    for k in 0..4: steps += walk(place['0' + k], place['0' + k + 1])?
    return steps

print(total() ?? -1)
