// expect: E0608 at 2
// rule: 05.7.1
fn twice(xs: List<Int>) -> List<Int>:
    var ys = xs
    ys.extend(xs)
    ys
