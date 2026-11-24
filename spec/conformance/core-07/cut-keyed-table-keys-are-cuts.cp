// expect: ok, output "1\n4 4 3"
// input: "3\n5 1 4 2\n"
// rule: 07.3.5
// A `[D.cuts]T` table keeps its cut keys through a generic `[D]T` param: only a real list
// gives `D = Int`, so `a.keys` is a `Range<D>` and a domain-keyed `LinearRmq` reads its cuts.
use std.sparse.{LinearRmq}
struct K<D>:
    keys: Range<D>
fn mk<D, T>(a: [D]T) -> K<D>:
    let keys = a.keys
    K<D>(keys)
let n: 1..=10 = read()
domain Ball = 0..n
type Cut = Ball.bounds
let w: [Cut]0..=100 = read()
let r = LinearRmq([Cut] b => w[b])
print(r.best(Ball.cuts.nth(0)..=Ball.cuts.nth(n)))
print(mk(w).keys.len, mk([Cut] b => w[b]).keys.len, r.argbest(Ball.cuts.nth(2)..=Ball.cuts.nth(n)).ord)
