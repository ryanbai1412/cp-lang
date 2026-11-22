// expect: ok, output "5 0 1"
// rule: 01.7.1
fn f(x: ?Int) -> Int = if let v = x, v > 2: v else: 0
let a: ?Int = 5
let b: ?Int = None
let big = a is Some(v) and v > 3
print(f(a), f(b), if big: 1 else: 0)
