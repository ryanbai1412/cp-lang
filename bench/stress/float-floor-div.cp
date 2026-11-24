// How many full boxes of b items a items fill, with a planted floating-point
// rounding: the fast fn divides as F64, whose 53-bit mantissa rounds a large
// count before the division.
// class: floating-point
let (a, b): (0..=1e18, 1..=1e9) = read()

fn boxes(a: Int, b: Int) -> Int = floor(F64(a) / F64(b))  // planted

@alt(boxes)
fn boxes_slow(a: Int, b: Int) -> Int = a / b

print(boxes(a, b))
