// expect: E0103 at 2
domain V = 1..=4
var a: [V]Int = 0
fn tail(xs: [V]Int, l: V) lends [V]Int = xs[l..]
