// expect: ok, output "5"
// rule: 08.2.4
// A `const` is the modulus in a variant's field type as in a struct's.
use std.mod.{Mod}
const P = 7
enum E:
    A(m: Mod<P>)
    B
let e = E.A(Mod(12))
match e:
    A(m) => print(m.v)
    B => print(0)
