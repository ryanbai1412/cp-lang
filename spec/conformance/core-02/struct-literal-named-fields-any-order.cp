// expect: ok, output "1 2"
// A struct is built with its fields named, in any order (round 23, H9).
struct P:
    a: Int
    b: Int
let p = P(b: 2, a: 1)
print(p.a, p.b)
