// expect: ok, output "1 2\n3 1"
// A MeldHeap of a type with no `+` pushes and pops; only `add_all` needs `Add`.
use std.heap.*
var h = MeldHeap.of([(at: 3, cnt: 1)])
h.push_all([(at: 1, cnt: 2)])
let a = h.pop()!
let b = h.pop()!
print(a.at, a.cnt)
print(b.at, b.cnt)
