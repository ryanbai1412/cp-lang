// expect: E0902
// rule: R03.1.10
// A constant below every value a range ending at a top-level `let` can start at is rejected
// when checked, as with constant ends: `?? 0` cannot be a `1..=m`.
let m: 1..=1e4 = 7
var x: ?1..=m = None
print(x ?? 0)
