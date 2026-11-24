// cco04p4 Return of Space Turtle: two points each loop a closed lattice orbit, one unit per
// minute, and are watched once a minute. Both are back at their starts together after the lcm
// of the two lap lengths (each at most 1e4), so walk one joint period. Within a stretch where
// neither turns, their offset moves in a straight line, so its closest point is a clamped
// quadratic minimum. That is O(turns in the period), not one step for each of up to 5e7
// minutes. A lap with no length stays put.
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
fn lap(
    x: Int,
    y: Int,
    moves: List<(d: Int, axis: Axis)>,
) -> (at: Pt, legs: List<(len: Int, dir: Pt)>):
    let unit = (a: Axis) => match a:
        X => Pt(1, 0)
        Y => Pt(0, 1)
    let legs = [(len: abs(mv.d), dir: unit(mv.axis) * Int(mv.d.sign)) for mv in moves if mv.d != 0]
    (at: Pt(x, y), legs: if legs.is_empty: [(len: 1, dir: Pt(0, 0))] else: legs)
var (s, t) = (lap(x1, y1, moves1.collect()), lap(x2, y2, moves2.collect()))
let period = lcm(s.legs.map(l => l.len).sum(), t.legs.map(l => l.len).sum())
var (leg_s, leg_t) = (0, 0)
var (left_s, left_t) = (s.legs[0].len, t.legs[0].len)
var best = s.at.dist2_128(t.at)
var minute = 0
while minute < period:
    // The offset over the next k minutes is r + v * c for c in 0..k; its square is convex in c.
    let k = min(left_s, left_t)
    let (r, v) = (s.at - t.at, s.legs[leg_s].dir - t.legs[leg_t].dir)
    let q = if v.norm2 == 0: 0 else: -dot(r, v) / v.norm2
    for c in [q - 1, q, q + 1]:
        best min= (r + v * min(max(c, 0), k - 1)).norm2_128
    s.at += s.legs[leg_s].dir * k
    t.at += t.legs[leg_t].dir * k
    (left_s, left_t, minute) = (left_s - k, left_t - k, minute + k)
    if left_s == 0:
        leg_s = (leg_s + 1) % s.legs.len
        left_s = s.legs[leg_s].len
    if left_t == 0:
        leg_t = (leg_t + 1) % t.legs.len
        left_t = t.legs[leg_t].len
print(sqrt(F64(best)), precision: 2)
