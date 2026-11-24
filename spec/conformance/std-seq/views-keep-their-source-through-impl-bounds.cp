// expect: ok, output "ok"
// rule: std.seq.rebase
// A view whose impl bound names its own source (`Rebased<Slice<List>>`, `Items<Range>`, `Mapped<List>`) still selects its impls, and its doc examples run.
use std.seq.*
assert([5, 6, 7][1..].rebase.nth(1) == 7)
let r = 0..3
assert(r.items.src == r)
let a = [1, 2]
assert(map(a, x => x * 2).src == a)
print("ok")
