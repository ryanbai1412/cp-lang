// expect: ok, output "6 9\n9 9"
// rule: R07.1.1, R05.5.3
// `for x in s` with `S: Seq<Int>` walks through the bound's `iter`; each instance picks the impl.
// `wrap` keeps a slice of `xs` that points at it, and its `s` field walks that same slice.
use std.seq.{Seq}
struct Holder<S>:
    s: S
    t: Int

fn total<S: Seq<Int>>(s: S) -> Int:
    var t = 0
    for x in s: t += x
    t

fn wrap<S: Seq<Int>>(s: S) -> Holder<S>:
    Holder(s, total(s))

let xs = [1, 2, 3, 4]
print(total([1, 2, 3]), total(xs[1..4]))
let h = wrap(xs[1..4])
var u = 0
for x in h.s: u += x
print(h.t, u)
