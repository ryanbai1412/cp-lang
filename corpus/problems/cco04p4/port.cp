// CCO 2004 Orbit, as far as it can be written today: it prints the least
// squared distance, because the prototype has no floats to take the root.
// Builds; on 800 script-made inputs the root of its answer, to two
// decimals, is what 5429989.cpp prints.
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
