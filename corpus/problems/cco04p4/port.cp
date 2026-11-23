// Source: CCO 2004 Stage 2 Day 2 P1 (Return of Space Turtle), https://dmoj.ca/problem/cco04p4
// Problem: two points each loop forever around a closed lattice orbit, given as a start and at
//   most 100 moves of |d| <= 100 units along X or Y, one unit per minute. Print the least
//   Euclidean distance between them over all whole minutes, to 2 decimals.
// Solution: expand both orbits into unit steps; after the lcm of the two lap lengths (at most
//   1e8) both are back at their starts together, so simulate that many minutes and keep the
//   least squared distance: O(lcm).
// Note: it prints the least squared distance, not its root to 2 decimals, because the
//   prototype has no floats. On 800 script-made inputs the root of its answer, to two
//   decimals, is what 5429989.cpp prints.
@tags("X", "Y")
enum Axis:
    X
    Y
type Pt = (x: Int, y: Int)

fn read_moves() -> List<Pt>:  // one lap as unit steps
    let n: 1..=100 = read()
    var moves: List<Pt> = []
    for _ in 0..n:
        let d: -100..=100 = read()
        let axis: Axis = read()
        let dir = if d > 0: 1 else: -1
        for _ in 0..abs(d):
            match axis:
                X => moves.push((dir, 0))
                Y => moves.push((0, dir))
    moves

var p: Pt = read()
let a = read_moves()
var q: Pt = read()
let b = read_moves()
var best = inf
if a.len > 0 and b.len > 0:
    for t in 0..lcm(a.len, b.len):  // after that both lists start again together
        best min= (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)
        let da = a[t % a.len]
        let db = b[t % b.len]
        p = (p.x + da.x, p.y + da.y)
        q = (q.x + db.x, q.y + db.y)
print(best)
