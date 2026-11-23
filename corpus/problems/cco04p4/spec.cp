// cco04p4 Return of Space Turtle: two points each loop a closed lattice orbit, one unit per
// minute, and are watched once a minute. Both are back at their starts together after the lcm
// of the two lap lengths (each at most 1e4), so walk one joint period and keep the closest
// approach: O(lcm), about 5e7 minutes at worst. A lap with no length stays put.
enum Axis = X | Y
fn lap() -> (at: Pt, steps: List<Pt>):
    let (x, y, m) = read<(Int, Int, 0..=100)>()
    let moves = read<[m](d: -100..=100, axis: Axis)>()
    let unit = (a: Axis) => match a:
        X => Pt(1, 0)
        Y => Pt(0, 1)
    let steps = [unit(mv.axis) * Int(mv.d.sign) for mv in moves for _ in 0..abs(mv.d)]
    (at: Pt(x, y), steps: if steps.is_empty: [Pt(0, 0)] else: steps)
var (s, t) = (lap(), lap())
var best = s.at.dist2_128(t.at)
for i in 0..lcm(s.steps.len, t.steps.len):
    s.at += s.steps[i % s.steps.len]
    t.at += t.steps[i % t.steps.len]
    best min= s.at.dist2_128(t.at)
print(sqrt(F64(best)), precision: 2)
