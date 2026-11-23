// expect: ok, output "-1 6 -1"
// rule: 02.4.1
// `x.and_then(f)` keeps `None` without calling `f` (here `f` would divide by zero), and flattens a `Some`.
let zero = 0
let n: ?Int = None
let s: ?Int = Some(3)
print(n.and_then(x => Some(x / zero)) ?? -1, s.and_then(x => if x > 1: Some(x * 2) else: None) ?? -1, s.and_then(x => if x > 5: Some(x) else: None) ?? -1)
