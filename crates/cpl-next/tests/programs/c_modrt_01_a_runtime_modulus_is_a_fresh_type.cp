// expect: ok, output "1 6\n5 3"
// rule: R03.5.1
// input: "7\n"
// `type Z = Mod(p)` runs where it is written, at the top level and in a fn: each `Z` is its own modulus, read once.
let p: Int = read()
type Z = Mod(p)
let a = Z(5)
print(a * Z(3), a + Z(1))
fn lift(m: Int, x: Int) -> Int:
    type W = Mod(m)
    W(x).v
print(lift(7, 12), lift(4, 7))
