// expect: ok, output "3 3 3"
// Each test case builds its own pool: frees in one case never leak a slot into the next.
use std.marks.{Arena, Id}
var out: List<Int> = []
for _ in 0..3:
    var a = Arena<Int>()
    let p = a.alloc(1)
    a.alloc(2)
    a.free(p)
    a.alloc(3)
    out.push(a.len + a.get(Id<Int>(0)) - 2)
print("{out[0]} {out[1]} {out[2]}")
