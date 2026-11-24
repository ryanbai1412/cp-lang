// expect: ok, output "17\n2"
// rule: std.subset.Subset.each
// A subset's folds take its members, as its `each` yields them, so they index a `[V]` table.
domain V = 1..=5
let s = Subset<V>.single(V(2)).with(V(4))
var w: [V]Int = 0
w[V(2)] = 10
w[V(4)] = 7
print(s.sum(of: v => w[v]))
print(s.max(by: v => w[v])!)
