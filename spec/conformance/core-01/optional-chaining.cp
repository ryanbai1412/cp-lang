// expect: ok, output "6 -1 1"
// rule: 01.8.6
fn inc(x: Int) -> Int = x + 1
fn g(x: ?Int) -> ?Int = inc(x?)
let a: ?Int = None
let p: ?(x: Int, y: Int) = (x: 1, y: 2)
print(g(5)!, g(a) ?? -1, p?.x ?? 0)
