// expect: ok, output "3 7"
// rule: 06.1.5
// An omitted parameter whose default is a bare operator (`over: M = (+)`) binds `M` to that
// operator exactly as passing `over: +` would, so `Monoid.of` sees `+`, not a fn value.
fn total<M>(xs: List<Int>, over: M = (+)) -> Int = xs.fold(Monoid<Int>.of(over))
let a = Action<Int, Int>(apply: (d, x, n) => x + d * n)
print(total([1, 2]), a.over.op(3, 4))
