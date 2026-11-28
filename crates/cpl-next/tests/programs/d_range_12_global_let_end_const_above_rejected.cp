// expect: E0902
// rule: R03.1.10
// A constant above every value a top-level `let` end can have, by the `let`'s declared range,
// is rejected when checked: `m` is at most `1e4`, so `0..m` never holds `1e4`.
let m: 1..=1e4 = 7
fn far() -> 0..m = 10000
print(far())
