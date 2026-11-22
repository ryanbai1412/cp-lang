// expect: ok, output "1 2 0"
// rule: 13.2.7
struct S: a: 0..=1, b: Int, c: 0..=1
print(S(1, 2, 0))
