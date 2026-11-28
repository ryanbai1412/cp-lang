// expect: E0902
// rule: 07.7.11 04.4.24
// A stride below 1 has no meaning, and `0` is a constant, so `step(0)` is rejected when compiled.
let e = [3, 1].step(0) // <- here
print(e.len)
