// expect: E0606
// rule: R10.3.7
// engines: next
let xs: List<?Int> = [Some(1), None]
print(xs)  // <- here
