// cco04p4 Return of Space Turtle: two points each loop a closed lattice orbit, one unit per
// minute, and are watched once a minute. Both are back at their starts together after the lcm
// of the two lap lengths (each at most 1e4), so walk one joint period and keep the closest
// approach: O(lcm), about 5e7 minutes at worst. A lap with no length stays put.
enum Axis = X | Y
// Each lap is closed and has positive length, as the statement promises (the
// accepted C++ indexes an empty lap out of bounds); gen.py's small sizes.
@gen fn input():
    for _ in 0..2:
        let (half, cap) = (rand(1, 12), rand(1, 5))
        let a = rand(0, half)
        var moves: List<(d: Int, axis: Str)> = []
        for (len, axis) in [(a, "X"), (half - a, "Y")]:
            for sign in [1, -1]:
                var left = len
                while left > 0:
                    moves.push((d: sign * min(cap, left), axis:))
                    left -= min(cap, left)
        for _ in 0..rand(0, 2):
            moves.push((d: 0, axis: if rand(0, 1) == 0: "X" else: "Y"))
        for i in 0..moves.len:
            moves.swap(i, rand(i, moves.len - 1))
        print(rand(-20, 20), rand(-20, 20), moves.len)
        for mv in moves:
            print(mv.d, mv.axis)
let (x1, y1, m1) = read<(Int, Int, 0..=100)>()
let moves1 = read<[m1](d: -100..=100, axis: Axis)>()
let (x2, y2, m2) = read<(Int, Int, 0..=100)>()
let moves2 = read<[m2](d: -100..=100, axis: Axis)>()
fn lap(x: Int, y: Int, moves: List<(d: Int, axis: Axis)>) -> (at: Pt, steps: List<Pt>):
    let unit = (a: Axis) => match a:
        X => Pt(1, 0)
        Y => Pt(0, 1)
    let steps = [unit(mv.axis) * Int(mv.d.sign) for mv in moves for _ in 0..abs(mv.d)]
    (at: Pt(x, y), steps: if steps.is_empty: [Pt(0, 0)] else: steps)
var (s, t) = (lap(x1, y1, moves1.collect()), lap(x2, y2, moves2.collect()))
var best = s.at.dist2_128(t.at)
for i in 0..lcm(s.steps.len, t.steps.len):
    s.at += s.steps[i % s.steps.len]
    t.at += t.steps[i % t.steps.len]
    best min= s.at.dist2_128(t.at)
print(sqrt(F64(best)), precision: 2)
