// expect: ok, output "6 0 0"
// `x.and_then(f)` (core 2.4) gives what `f` gives, and skips `f` on `None`.
let half = (x: Int) => if x > 1: Some(x * 2) else: None
let a: ?Int = Some(3)
let b: ?Int = Some(1)
let c: ?Int = None
print(a.and_then(half) ?? 0, b.and_then(half) ?? 0, c.and_then(half) ?? 0)
