// expect: E0603
// rule: 05.7.8
domain V = 1..=4
fn tail(xs: [V]Int, l: V) -> [V]Int = xs[l..] // <- here
