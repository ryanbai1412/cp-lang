// expect: E0603
// rule: 05.7.10
var kept = [0]
fn f(xs: List<Int>):
    kept = xs // <- here
