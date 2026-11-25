// expect: R0201
// A narrower range fits a wider one with no check; the other way is checked.
let c: 0..=3 = 2
var w: 0..=10 = c
fn f(a: 0..=3) -> 0..=10 = a
print(f(c), w)
w = 7
var z: 0..=3 = w // <- here
