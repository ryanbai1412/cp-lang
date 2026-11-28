// expect: ok, output "5"
// rule: 08.2.4
// A `const` is the modulus in a struct field's type as in a fn's.
use std.mod.{Mod}
const P = 7
struct S:
    m: Mod<P>
let s = S(Mod(12))
print(s.m.v)
