// expect: R0202
// rule: R09.3
// A range type's end that overflows an `Int` is the run-time overflow, not a constant.
let x: 0..=2 ** 63 = 0
print(x)
