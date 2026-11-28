// expect: ok, output "65 A 7 7 13 0 1"
// rule: R04.2.28
// `Int(x)`, `Char(x)` and `U64(x)` are core's `Into` impls; a position gives its integer, an enum its position.
domain V = 10..20
enum Color:
    Red
    Green
let c = 'A'
let u = U64(7)
print(Int(c), Char(65), u, Int(u), Int(V(13)), Int(Color.Red), Int(Color.Green))
