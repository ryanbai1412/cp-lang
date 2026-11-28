// expect: E0302
// rule: 08.2.4
// engines: next
// A `const` worked out through a struct whose field's type names it waits on itself.
use std.mod.{Mod}
struct S:
    m: Mod<P>
fn f() -> Int:
    S(Mod(3)).m.v + 5
const P = f() // <- here
print(P)
