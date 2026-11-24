// expect: E0302
// rule: 01.5.1
let s = 3
let b: Int = s & (s << 1) == 0 // <- here
print(b)
