// expect: E0302 at 3
// rule: 04.8.10
// A domain with constant bounds writes its count, so a list literal of the wrong length for an array over it is a compile error, as for `[0..3]Int`.
domain V = 0..3
let b: [V]Int = [0, 2]
print(b)
