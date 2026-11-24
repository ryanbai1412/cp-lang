// expect: L0608
// rule: 07.9.6
// A range's own `sum` is closed form but still a method, so reading it without `()` warns (decision 140).
let r = 1..=10
print(r.sum) // <- here
