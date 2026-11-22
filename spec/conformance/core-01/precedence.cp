// expect: ok, output "-9 1"
// rule: 01.5.1
let x = 3
let s = 2
print(-x ** 2, if s & (s << 1) == 0: 1 else: 0)
