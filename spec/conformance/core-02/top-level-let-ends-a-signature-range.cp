// expect: ok, output "6 5"
// rule: 02.10.3
// A top-level `let` ends a range type: the type is one range for the whole run.
let m: 1..=10 = 5
fn twice(c: 1..=m) -> Int = c * 2
print(twice(3), m)
