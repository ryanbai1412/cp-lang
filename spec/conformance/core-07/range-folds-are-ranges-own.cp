// expect: ok, output "1000000000000000000 1 1000000000000000000 1000000000000000000"
// rule: 07.9.6
// A range's own `len`, `min()`, `max()` and `count()` are O(1); a walk of 1e18 elements would time out.
let r = 1..=1e18
print(r.len, r.min()!, r.max()!, r.count())
