// IOI 2015 Day 1 P1, Boxes with Souvenirs
// original: corpus/submissions/5723345.cpp
@grader
fn delivery(n: Int, k: Int, l: Int, pos: [0..n]Int) -> Int:
    // cw[i]: the first i teams served by clockwise round trips, k teams a trip;
    // ccw[i]: teams i.. served by anticlockwise round trips
    var cw: [0..=n]Int = 0
    for i in 1..=n: cw[i] = 2 * pos[i - 1] + cw[max(0, i - k)]
    var ccw: [0..=n]Int = 0
    for i in (0..n).rev: ccw[i] = 2 * (l - pos[i]) + ccw[min(n, i + k)]

    // split the teams at i, with at most one full lap taking the k teams after it
    return min(cw[i] + min(ccw[i], l + ccw[min(n, i + k)]) for i in 0..=n)!
