// expect: R0401
// rule: 07.7.11
// A stride below 1 has no meaning, so `step(0)` traps where it is made.
let e = [3, 1].step(0) // <- here
print(e.len)
