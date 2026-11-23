// expect: ok, output "1 3"
// A key of `V.ranges` is non-empty, so `first`/`last` are `V`, not `?V` (lib/search-dp.md, decision 322).
domain V = 0..5
let r: V.ranges = V(1)..=V(3)
print(Int(r.first), Int(r.last))
