// CCO 2004 P4 Gator Chase-ish: two points each repeat their own cycle of moves forever.
// Report the closest they ever get. Their joint state repeats after lcm of the cycle lengths.
enum Axis:
    @tag("X") X
    @tag("Y") Y
type Pt = (x: Int, y: Int)

// The points one walk visits, one per unit step, before it starts over.
fn orbit() -> List<Pt>:
    var at: Pt = read()
    let n: 0..=100 = read()
    let moves: [n](d: -100..=100, axis: Axis) = read()
    var seen: List<Pt> = []
    for m in moves:
        let unit = if m.d > 0: 1 else: -1
        for _ in 0..abs(m.d):
            seen.push(at)
            at = match m.axis:
                X => (at.x + unit, at.y)
                Y => (at.x, at.y + unit)
    if seen.is_empty: seen.push(at)
    seen

let (a, b) = (orbit(), orbit())
fn gap(p: Pt, q: Pt) -> Int = (p.x - q.x) ** 2 + (p.y - q.y) ** 2
print(sqrt(F64(min(gap(a[t % a.len], b[t % b.len]) for t in 0..lcm(a.len, b.len))!)), precision: 2)
