// expect: ok, output "1 1 7"
// rule: 03.2.9
// A `List<N>` stores its elements apart, and an `Id<P>` handle into a
// `PArena<P>` is a number, so both link nodes of a struct to each other.
use std.persistent.{PArena}
use std.marks.{Id}
struct N:
    k: Int
    kids: List<N>
struct P:
    k: Int
    up: ?Id<P>
let t = N(1, [N(2, [])])
var nodes = PArena<P>()
let root = nodes.alloc(P(7, None))
let leaf = nodes.alloc(P(3, Some(root)))
print(t.k, t.kids.len, nodes.get(nodes.get(leaf).up!).k)
