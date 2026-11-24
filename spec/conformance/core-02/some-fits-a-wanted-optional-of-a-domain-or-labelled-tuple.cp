// expect: ok, output "1 2 1 1 4"
// rule: 02.4.2
// `Some(e)` fits a wanted `?D` or `?(to: D, w: Int)` the way `e` fits the payload: a labelled tuple of a domain pair returned in `Some`, `Some(p)` into a `[V]?V` slot, and `Some(xs[k])` in a value `if`.
domain W = 0..3
domain Tail = (W, W)
fn lay(t: W) -> ?(to: Tail, w: Int) = if t < 2: Some((to: (t, W.last!), w: 1)) else: None
let n = 4
domain V = 0..n
var up: [V]?V = None
let p: V = 1
up[p] = Some(p)
fn at(xs: List<Int>, k: Int) -> ?Int = if k < xs.len: Some(xs[k]) else: None
print(lay(1)!.to.0, lay(0)!.to.1, up[p]!, at([7, 1], 1)!, at([4], 0) ?? 0)
