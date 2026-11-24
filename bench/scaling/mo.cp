// n: 4096
use std.mo.{mo, mo_order, mo_order_hilbert, mo_rollback, mo_steps, mo_with_updates}
let n: Int = read()
domain P = 0..n
var qs: List<Range<P>> = []
var tq: List<(range: Range<P>, time: Int)> = []
for i in 0..n:
    let a = (i * 7919) % n
    let b = (i * 104729 + 17) % n
    qs.push(P(min(a, b))..P(max(a, b)))
    tq.push((range: P(min(a, b))..P(max(a, b)), time: (i * 31) % (n + 1)))
let ups = [i % 7 for i in 0..n]
var cnt = 0
let add: P -> () = x => cnt += 1 // within: mo.mo
let remove: P -> () = x => cnt -= 1 // within: mo.mo
let answer: Int -> Int = t => cnt // within: mo.mo
let mark: () -> Int = () => cnt // within: mo.mo_rollback
let radd: P -> () = x => cnt += 1 // within: mo.mo_rollback
let ranswer: Int -> Int = t => cnt // within: mo.mo_rollback
let undo: Int -> () = m => cnt = m // within: mo.mo_rollback
let reset: () -> () = () => cnt = 0 // within: mo.mo_rollback
let uadd: P -> () = x => cnt += 1 // within: mo.mo_with_updates
let uremove: P -> () = x => cnt -= 1 // within: mo.mo_with_updates
let apply: (Int, Range<P>) -> () = (u, r) => cnt += u // within: mo.mo_with_updates
let unapply: (Int, Range<P>) -> () = (u, r) => cnt -= u // within: mo.mo_with_updates
let uanswer: Int -> Int = t => cnt // within: mo.mo_with_updates
var s = 0
for _ in 0..20:
    s += mo_order(qs).len // timed: mo.mo_order
    s += mo_order_hilbert(qs).len // timed: mo.mo_order_hilbert
for i in 0..400:
    s += mo_steps(P(0)..P(n / 2 + i % 2), P(n / 2)..P(n - 1)).len // timed: mo.mo_steps
for _ in 0..3:
    s += mo(qs, add:, remove:, answer:).len // timed: mo.mo
    s += mo_rollback(qs, mark:, add: radd, answer: ranswer, undo:, reset:).len // timed: mo.mo_rollback
    s += mo_with_updates(tq, ups, add: uadd, remove: uremove, apply_update: apply, undo_update: unapply, answer: uanswer).len // timed: mo.mo_with_updates
print(s, cnt)
