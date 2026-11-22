// cco04p4: two walkers repeat their scripts of unit steps forever. Both return to the same
// phase after lcm(len_s, len_t) steps, so the closest approach happens within one joint period.
fn walker() -> (at: Pt, steps: List<Pt>):
    let (x, y, n) = read<(Int, Int, 0..=10_000)>()
    let moves = read<[n](d: Int, axis: Char)>()
    let unit = (m: (d: Int, axis: Char)) => if m.axis == 'X': Pt(Int(sign(m.d)), 0) else: Pt(0, Int(sign(m.d)))
    (Pt(x, y), moves.flat_map(m => [unit(m) for _ in 0..abs(m.d)]).collect())
var (s, t) = (walker(), walker())
var best = s.at.dist(t.at)
for i in 0..lcm(s.steps.len, t.steps.len):
    s.at += s.steps[i % s.steps.len]
    t.at += t.steps[i % t.steps.len]
    best min= s.at.dist(t.at)
print(best, precision: 2)
