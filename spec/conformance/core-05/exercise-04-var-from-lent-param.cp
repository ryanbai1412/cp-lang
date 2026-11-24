// expect: E0608
// gap: compiler 2026-11-24T05Z
// rule: 05.7.1
fn twice(xs: List<Int>) -> List<Int>:
    var ys = xs // <- here
    ys.extend(xs)
    ys
