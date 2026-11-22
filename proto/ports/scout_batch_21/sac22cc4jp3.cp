// SAC 2022 Code Challenge 4 Junior P3: n points and a direction, given as a
// whole number of degrees counter-clockwise from the x axis. Print the
// number of the point whose direction from the origin is closest to it
// (the first such point).
//
// The gap between two directions is the short way round the circle. The
// first point wins ties because only a strictly smaller gap replaces it;
// the gaps are compared as computed, with no tolerance, as in the original.
// original: corpus/submissions/4416076.cpp
use real.float.{PI}

let n: Int = read()
let aim: Int = read()

// the angle between two directions in degrees, 0..=180
fn gap(a: F64, b: F64) -> F64:
    var turn = a - b
    while turn < 0.0: turn += 360
    while turn >= 360.0: turn -= 360
    min(turn, 360 - turn)

var best: ?(gap: F64, id: Int) = none
for id in 1..=n:
    let x: F64 = read()
    let y: F64 = read()
    let g = gap(F64(aim), atan2(y, x) * 180 / PI)
    if best == none or g < best!.gap: best = (gap: g, id:)
print(best!.id)
