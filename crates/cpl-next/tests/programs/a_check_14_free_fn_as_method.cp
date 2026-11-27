// expect: E0203
// rule: 06.1.2
// A fn without `self` is not a method; the error writes the free call `twice(k, 2)`.
fn twice(a: Int, b: Int) -> Int = a * b
let k = 4
print(k.twice(2))  // <- here
