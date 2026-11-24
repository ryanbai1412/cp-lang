// expect: E0608
// rule: 05.7.1
fn twice(xs: List<Int>) -> List<Int>:
    var ys = xs // <- here
    ys.extend(xs)
    ys
