// expect: ok, output "3 9"
// rule: R05.5.3
// `h.s` is a slot lent from `xs`; walking it calls the `iter` of the lent slice, not of a copy.
use std.seq.{Sized}
struct Holder<S>:
    s: S
    t: Int

fn wrap<S: Sized>(s: S) -> Holder<S> = Holder(s, s.len)

let xs = [1, 2, 3, 4]
let h = wrap(xs[1..4])
var u = 0
for x in h.s: u += x
print(h.t, u)
