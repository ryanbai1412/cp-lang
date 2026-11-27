// expect: ok, output "65 A 7 7 13 2 5"
// rule: R04.2.28
// `Int(x)`, `Char(x)` and `U64(x)` are core's `Into` impls; a position gives its integer, an enum its value.
domain V = 10..20
enum Color:
    Red = 2
    Green = 5
let c = 'A'
let u = U64(7)
print(Int(c), Char(65), u, Int(u), Int(V(13)), Int(Color.Red), Int(Color.Green))
