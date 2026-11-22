// expect: ok, output "1 2 3"
// rule: 02.6.7 03.2.8
struct S: a: 0..=9, b: Int, c: 0..=9
let s = S(1, 2, 3)
print(s)
