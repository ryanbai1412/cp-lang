struct S(a: 0..=1_000_000, b: Int)
let k: Int = read()
var s = S(0, 1)
s.a = k
print(s.a)
