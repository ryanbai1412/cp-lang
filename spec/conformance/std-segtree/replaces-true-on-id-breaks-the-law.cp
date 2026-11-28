// expect: R0401 (checked)
// rule: std.algebra.Action
// `replaces` must not hold on `id`: `apply(id, a, 1)` is `a`, which depends on `a`. `laws_hold`
// stops on a predicate that claims it, so a fold never trusts `id` to set a span.
use std.algebra.{Action}
let good = Action<Int, ?Int>(None, (nf, of) => nf ?? of, (f, v, span) => f ?? v, over: max, replaces: f => f != None)
assert laws_hold(good, (T: [1, 5, -3], F: [None, Some(2), Some(7)]))
let bad = Action<Int, ?Int>(None, (nf, of) => nf ?? of, (f, v, span) => f ?? v, over: max, replaces: _ => true)
assert laws_hold(bad, (T: [1, 5, -3], F: [None, Some(2), Some(7)])) // <- here
