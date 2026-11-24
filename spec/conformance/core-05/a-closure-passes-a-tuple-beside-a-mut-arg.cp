// expect: ok, output "2 3\n2 3\n1 0 7"
// rule: 05.3.3 05.10.4
// A closure that takes a `mut` tuple and a plain tuple hands both on whole: the callee reads the plain one as passed and its write to the `mut` one lands in the caller's variable.
use std.algebra.{Action}
fn affine_onto(mut old: (Int, Int), new: (Int, Int)):
    old = (new.0 * old.0, new.0 * old.1 + new.1)
fn show(mut old: (Int, Int), x: Int):
    print(old.0, old.1, x)
    old = (old.0, old.1)
let comp = (nf: (Int, Int), of: (Int, Int)) => (nf.0 * of.0, nf.0 * of.1 + nf.1)
let app = (f: (Int, Int), v: Int, span: Int) => f.0 * v + f.1 * span
let good = Action<Int, (Int, Int)>(id: (1, 0), compose: comp, apply: app, compose_in: (mut o: (Int, Int), nf: (Int, Int)) => affine_onto(mut o, nf))
var h = (1, 0)
good.compose_onto(mut h, (2, 3))
print(h.0, h.1)
let c = (mut o: (Int, Int), nf: (Int, Int)) => affine_onto(mut o, nf)
var k = (1, 0)
c(mut k, (2, 3))
print(k.0, k.1)
let d = (mut o: (Int, Int), x: Int) => show(mut o, x)
var m = (1, 0)
d(mut m, 7)
