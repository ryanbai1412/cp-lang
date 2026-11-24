// fixed: components
use std.algebra.{MaxSub, MaxSubAt, Semigroup, point_action, MinCount, MaxCount, Affine, Group, Action, each, fold, pow, desc, minmax, cmp, add}
let n: Int = read()
let xs = [i * 7 % 1000 for i in 0..n]
var acc = 0
for i in 0..8 * n:
    let m = Monoid<Int>.of(+) // timed: algebra.Monoid.of
    acc += m.monoid().op(i, 1) // timed: algebra.Monoid.monoid
    let cm = m.commutative() // timed: algebra.Monoid.commutative
    acc += cm.monoid().op(i, 1) // timed: algebra.Comm.monoid
    let act = Action<Int, Int>(apply: (d, x, span) => x + d * span) // timed: algebra.Action.new
    acc += act.apply(1, i, 2)
    acc += Action<Int, Int>.of(add, +).apply(2, i, 3) // timed: algebra.Action.of
    if desc(i) < desc(acc): acc += 1 // timed: algebra.desc
    let s = MaxSub.join(MaxSub.of(i % 7 - 3), MaxSub.zero) // timed: algebra.MaxSub.zero
    let t = MaxSub.of(i % 5 - 2) // timed: algebra.MaxSub.of
    let u = MaxSub.join(s, t) // timed: algebra.MaxSub.join
    acc += u.scale(2).try_best ?? 0 // timed: algebra.MaxSub.scale
    acc += u.try_best ?? 0 // timed: algebra.MaxSub.try_best
    acc += MaxSub.monoid().op(u, t).try_best ?? 0 // timed: algebra.MaxSub.monoid
    acc += Semigroup((a: Int, b: Int) => a - b).opt().op(Some(i), Some(1)) ?? 0 // timed: algebra.Semigroup.opt
    acc += Group<Int>.of(+).inv(i) // timed: algebra.Group.of
    acc += point_action(id: 0, compose: (a, b) => a + b, apply: (f, x) => x + f, over: max).apply(3, i, 4) // timed: algebra.point_action
    let c = MinCount.of(i % 3) // timed: algebra.MinCount.of
    let c2 = MinCount.join(c, MinCount<Int>.zero) // timed: algebra.MinCount.zero
    acc += MinCount.join(c2, c).count // timed: algebra.MinCount.join
    acc += MinCount.monoid().op(c, c2).count // timed: algebra.MinCount.monoid
    let d = MaxCount.of(i % 3) // timed: algebra.MaxCount.of
    let d2 = MaxCount.join(d, MaxCount<Int>.zero) // timed: algebra.MaxCount.zero
    acc += MaxCount.join(d2, d).count // timed: algebra.MaxCount.join
    acc += MaxCount.monoid().op(d, d2).count // timed: algebra.MaxCount.monoid
    let f = Affine.then(Affine<Int>.id(), Affine(2, i % 5)) // timed: algebra.Affine.id
    let g = Affine.then(f, Affine(1, 1)) // timed: algebra.Affine.then
    acc += g.eval(i % 11) // timed: algebra.Affine.eval
    acc += Affine.monoid().op(f, g).eval(1) // timed: algebra.Affine.monoid
    if cmp(i, acc) == Ordering.Less: acc += 1 // timed: algebra.cmp
    acc += minmax(i, acc % 5).0 // timed: algebra.minmax
    acc += each(+, max).monoid<(Int, Int)>().op((i, 5), (2, 3)).0 // timed: algebra.Each.monoid
    acc += fold(each(+, max), [(i, 5), (2, 3)]).0 // timed: algebra.each
    let o = (a: Int, b: Int) => a * 10 + b
    acc += o.opt().op(Some(i % 9), None) ?? 0 // timed: algebra.opt
    acc %= 1000000007
domain D = 0..8
for i in 0..8 * n:
    let a = MaxSubAt.of(D(i % 8), i % 7 - 3) // timed: algebra.MaxSubAt.of
    let b = MaxSubAt.join(a, MaxSubAt<D>.zero) // timed: algebra.MaxSubAt.zero
    let c = MaxSubAt.join(b, MaxSubAt.of(D((i + 1) % 8), 2)) // timed: algebra.MaxSubAt.join
    acc += c.try_best ?? 0 // timed: algebra.MaxSubAt.try_best
    if c.try_at != None: acc += 1 // timed: algebra.MaxSubAt.try_at
    acc += MaxSubAt<D>.monoid().op(a, c).try_best ?? 0 // timed: algebra.MaxSubAt.monoid
for r in 0..640:
    acc += fold(+, xs) // timed: algebra.fold
for i in 0..4 * n:
    acc ^= pow(3, n + i, op: +) // timed: algebra.pow
print(acc)
