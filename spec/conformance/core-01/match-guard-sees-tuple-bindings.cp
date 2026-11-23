// expect: ok, output "2"
// rule: 01.10
// A guard sees the names its pattern binds, inside a tuple pattern too.
let r = (Some(1), Some(3))
let x = match r:
    (Some(lo), Some(hi)) if lo < hi => hi - lo
    _ => 0
print(x)
