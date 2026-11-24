// expect: ok, output "2 1"
// rule: std.tree.Rooted.new
// `Rooted.new` over a domain declared from a runtime size infers its fresh order domain and keys depths by `P`.
use std.tree.{Rooted}
let n = 3
domain P = 0..n
let g: [P]List<P> = [[P(1)], [P(0), P(2)], [P(1)]]
let t = Rooted.new(g, P(0))
print(t.depth(P(2)), t.depth(P(1)))
