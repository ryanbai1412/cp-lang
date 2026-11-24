// Source: WOSS Dual Olympiad 2021 S3 (Late for Class!), https://dmoj.ca/problem/wossoly21s3
// limits: 2000 ms, 512 MB
// Problem: an R x C grid (R, C <= 200) of '.', '#' walls, marks 0-4 and letter portals: stepping
//   onto capital X teleports you for free to its small x, and small letters can't be entered.
//   Walk 0 -> 1 -> 2 -> 3 -> 4 in order (4-neighbour steps, 1 s each); print the fewest seconds,
//   or -1 if some class can't be reached.
// Solution: the legs are independent, so add four shortest paths. Each is a std bfs over cells
//   where a step onto q lands at land(q): the small letter for a capital, nowhere for '#' or a
//   small letter, q itself otherwise. O(RC) per leg; an unreachable leg makes the total -1.
// original: corpus/submissions/4224399.cpp
// Bounds: n, m: 1..=200
let (n, m): (1..=200, 1..=200) = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let school: [Row, Col]Char = read()

var place = Map<Char, Cell>()
for r in Row:
    for c in Col: place[school[r, c]] = (r, c)

// where a step onto q really puts you
fn land(q: Cell) -> ?Cell:
    let ch = school[q]
    if ch == '#' or ch.is_lower: None elif ch.is_upper: place[ch.lower] else: q

fn walk(from: Char, to: Char) -> ?Int = bfs(p => p.adj4.filter_map(land), from: [place[from]], until: place[to])[place[to]]

let legs = [walk('0' + k, '0' + k + 1) for k in 0..4].all_some()
print(if let steps = legs: steps.sum() else: -1)
