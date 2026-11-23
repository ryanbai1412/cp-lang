// WOSS Dual Olympiad 2021 S3, Fleeing Through the School: walk a grid
// through the periods 0, 1, 2, 3, 4 in order. Stepping onto a capital
// letter carries you to its small letter, which cannot be stepped onto
// directly. Print the fewest steps, or -1.
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
