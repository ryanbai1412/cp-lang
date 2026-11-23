// expect: ok
// rule: 07.9.6
// tool: build -o -
// lacks: "for (ll t"
// A range's own `min()` and `max()` are O(1) (R07.9.6), so the C++ computes them from the ends; a walk of 1e18 values only finishes when the optimizer folds it away.
let r = 1..=1e18
print(r.min()!, r.max()!)
