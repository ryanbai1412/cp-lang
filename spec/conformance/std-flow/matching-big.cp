// expect: ok, output "500\n500\nYES"
use std.flow.{max_matching, Matching}
domain L = 0..500
domain R = 0..500
var adj: [L]List<R> = []
for l in L:
    adj[l].push(R(l.ord))
    adj[l].push(R((l.ord * 7 + 3) % 500))
let m = max_matching(adj)
print(m.size)
var o = Matching<L, R>.new()
for l in L:
    for r in adj[l]: o.add(l, r)
var grown = 0
for l in L:
    if o.augment(l): grown += 1
print(grown)
var cov = 0
let c = o.cover()
for l in L:
    if c.left[l]: cov += 1
for r in R:
    if c.right[r]: cov += 1
print(cov == 500)
