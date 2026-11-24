// Source: COCI 2010/2011 Contest 7 #4, Postar, https://dmoj.ca/problem/coci10c7p4
// Problem: an N x N town (N <= 50) of cells P (one post office), K (houses, at least one) and
//   '.', each with a height < 1e6. Starting from P and moving to any of the 8 neighbours,
//   visit every K and return. Print the smallest possible (highest - lowest) height visited.
// Solution: Two pointers over the sorted heights: raising the floor never lowers the needed
//   ceiling. For each floor, raise the ceiling until a flood fill from P through cells with
//   heights inside [floor, ceiling] reaches every K. O(N^2) window steps, each an O(N^2)
//   fill: O(N^4).
// original: corpus/submissions/5245379.cpp
// Bounds: n: 2..=50; high: 1..=1e6
let n: 2..=50 = read()
domain Row = 1..=n
domain Col = 1..=n
let town: [Row, Col]Char = read()
let high: [Row, Col]1..=1e6 = read()
let cells = [(r, c) for r in Row for c in Col]
let post = cells.find(p => town[p] == 'P')!

// Can he deliver everything standing only on ground between lo and hi?
fn delivers(lo: Int, hi: Int) -> Bool:
    fn inside(p: (Row, Col)) -> Bool = lo <= high[p] and high[p] <= hi
    if not inside(post): return false
    var seen: [Row, Col]Bool = false
    seen[post] = true
    var todo = [post]
    while let p = todo.pop():
        for q in p.adj8:
            if inside(q) and not seen[q]:
                seen[q] = true
                todo.push(q)
    cells.all(p => town[p] != 'K' or seen[p])

// Raising the floor never lowers the ceiling it needs, so both only move up.
let levels = cells.map(p => high[p]).sorted()
var best = levels.last! - levels.first!
var top = 0
for floor in 0..levels.len:
    top max= floor
    while top < levels.len and not delivers(levels[floor], levels[top]): top += 1
    if top == levels.len: break
    best min= levels[top] - levels[floor]
print(best)
