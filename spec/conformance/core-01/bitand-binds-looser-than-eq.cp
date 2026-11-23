// expect: E0302 at 2
// rule: 01.5.1
let s = 3
let b: Int = s & (s << 1) == 0
print(b)
