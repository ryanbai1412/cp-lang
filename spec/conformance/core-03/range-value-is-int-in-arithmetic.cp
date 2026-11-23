// expect: ok, output "5 -3"
// rule: 03.1.7
// A value of a range type is an `Int` in arithmetic and in `min`/`max`.
struct S(l: 0..=1e9)
let s = S(l: 5)
print(max(2 - 10, s.l), min(s.l, -3))
