// expect: R0201
// rule: R02.10.3
// A store into a range type that ends at a top-level `let` is checked against its value.
let m: 1..=10000 = 3
fn keep(c: 1..=m) -> Int = c
print(keep(3))
print(keep(4))
