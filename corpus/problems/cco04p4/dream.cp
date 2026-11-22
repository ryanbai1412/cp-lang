# CCO 2004 P4 Gator Chase-ish: two points each repeat their own cycle of moves forever.
# Report the closest they ever get. Their joint state repeats after lcm of the cycle lengths.
let walk = read (start: (x: Int, y: Int), moves: List<(dx: Int, dy: Int)>)
let (a, b) = (read walk, read walk)
fn path(w) = w.moves.ring.scan(w.start, (p, m) => (x: p.x + m.dx, y: p.y + m.dy))
let period = lcm(a.moves.len, b.moves.len)
print(zip(path(a), path(b)).take(period + 1).min(of: (p, q) => hypot(p.x - q.x, p.y - q.y)), digits: 2)
