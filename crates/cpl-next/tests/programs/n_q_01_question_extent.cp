// expect: ok, output "6 -1\n3 0\n4 -1\n1 2\n12 -1"
// `x?` reads an optional's value; when it is `None` the nearest extent (a statement right-hand
// side, the left of `??`, a `fn ... =` body) is `None`. Operands after a `None` aren't read.
fn inc(x: Int) -> Int = x + 1
fn g(x: ?Int) -> ?Int = inc(x?)
fn h(a: ?Int, b: ?Int) -> ?Int:
    return a? * b?
var looks = 0
fn look(x: ?Int) -> ?Int:
    looks += 1
    return x
print(g(5)!, g(None) ?? -1)
let none: ?Int = None
let s = look(Some(1))? + look(Some(2))?
let t = look(none)? + look(Some(2))?
print(s!, looks - 3)
let xs = [10, 20, 30, 4]
let i: ?Int = Some(3)
let p: ?(x: Int, y: Int) = (x: 1, y: 2)
var v: ?Int = None
v = xs[i?]
print(v!, (xs[none?] + 1) ?? -1)
print(p?.x ?? 0, t ?? 2)
print(h(Some(3), Some(4))!, h(None, Some(4)) ?? -1)
