// expect: E0503
// rule: 01.10.2
// has: "-> lends"
domain V = 1..=4
var a: [V]Int = 0
fn tail(xs: [V]Int, l: V) lends [V]Int = xs[l..] // <- here
