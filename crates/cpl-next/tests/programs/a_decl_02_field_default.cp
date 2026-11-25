// expect: ok, output "12"
// rule: R02.6
// A struct field default.
struct P:
    x: Int
    y: Int = 2
let p = P(1)
print(p.x * 10 + p.y)
