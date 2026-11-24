// An argument read in place from an arena element is copied into a call
// that grows the arena, which would leave a reference dangling.
use std.marks.{Arena, Id}
struct Node(l: Id<Node>, r: Id<Node>)
var seg = Arena<Node>()
let leaf = seg.alloc(Node(Id(0), Id(0)))
fn grow(a: Id<Node>, d: Int) -> Id<Node>:
    if d == 0: return a
    seg.alloc(Node(grow(seg[a].l, d - 1), grow(seg[a].r, d - 1)))
let top = grow(leaf, 8)
print(yes(top == leaf), yes(seg[top].l == leaf))
