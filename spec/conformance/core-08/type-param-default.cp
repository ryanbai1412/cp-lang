// expect: ok, output "x 3"
// rule: 08.2.5
struct Two<A, B = Int>: a: A, b: B
let p: Two<Str> = Two("x", 3)
print(p.a, p.b)
