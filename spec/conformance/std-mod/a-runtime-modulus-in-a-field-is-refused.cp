// expect: E0302
// rule: 08.2.4
// engines: next
// A field's type argument read as the program runs is no constant: `type Z = Mod(p)` is that spelling.
use std.mod.{Mod}
let p = read<Int>()
struct S:
    m: Mod<p> // <- here
let s = S(Mod(12))
print(s.m.v)
