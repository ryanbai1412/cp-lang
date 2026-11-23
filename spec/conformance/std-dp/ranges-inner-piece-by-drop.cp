// expect: ok, output "2 2 1 3"
// The inner pieces of a `V.ranges` key are `r.drop(1)` and `r.drop_last(1)` (decision 322).
domain V = 0..5
let r: V.ranges = V(1)..=V(3)
let inner = r.drop(1)
let outer = r.drop_last(1)
print(inner.len, outer.len, Int(outer.first!), Int(inner.last!))
