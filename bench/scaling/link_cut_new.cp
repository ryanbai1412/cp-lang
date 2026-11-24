// n: 2048
use std.tree.{LinkCut, LazyLinkCut}
let n: Int = read()
let act = Action<Int, Int>(apply: (d, x, k) => x + d * k)
domain W = 0..32 * n
let wvals: [W]Int = [W] w => w.ord % 10
var s = 0
for _ in 0..40:
    s += LinkCut(+, a: Some(wvals)).get(W(0)) // timed: tree.LinkCut.new
    var z = LazyLinkCut(act, a: Some(wvals)) // timed: tree.LazyLinkCut.new
    s += z.get(W(0))
print(s)
