// expect: E0302 at 4
// rule: 04.8.10
// A list literal compared with an array takes the array's type, so one of the wrong length is a compile error, never a NO or a trap.
domain V = 0..3
var a: [V]Int = 0
print(yes(a == [0, 2]))
