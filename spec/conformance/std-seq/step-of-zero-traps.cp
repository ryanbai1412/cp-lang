// expect: R0401 at 4
// rule: 07.7.11
// A stride below 1 has no meaning, so `step(0)` traps where it is made.
let e = [3, 1].step(0)
print(e.len)
