// CCO 2004 Orbit: two craft start somewhere on a grid and repeat their own
// list of moves ("3 X": three unit steps along x) for ever. Print the
// closest they get before both lists start again together.
// WISH F1

@tags("X", "Y")
enum Axis:
    X
    Y

fn read_moves() -> List<Pt>:  // one lap as unit steps
    let n: 1..=100 = read()
    let legs: [n](d: -100..=100, axis: Axis) = read()  // WISH G25: a record with an enum field
    let unit = (leg) => match leg.axis:                // WISH G19: match as an expression
        X => Pt(sign(leg.d), 0)                        // WISH G20: sign
        Y => Pt(0, sign(leg.d))
    [unit(leg) for leg in legs for _ in 0..abs(leg.d)]

let start_a: Pt = read()
let a = read_moves()
let start_b: Pt = read()
let b = read_moves()

// where a craft is at every tick: the running sum of its moves, for ever
let path_a = scan(a.cycle, +, from: start_a)  // WISH F2: scan; G21: cycle
let path_b = scan(b.cycle, +, from: start_b)
let ticks = lcm(a.len, b.len)
let closest = min((p - q).norm2 for (p, q) in zip(path_a, path_b).take(ticks))  // WISH G21, G15
print(sqrt(closest), digits: 2)                                                 // WISH F15, G4
