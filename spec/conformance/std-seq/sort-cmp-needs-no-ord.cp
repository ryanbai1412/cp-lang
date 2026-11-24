// expect: ok, output "1 2\n1 3"
// `cmp:` orders elements with no `Ord`: neither `sort` nor `sorted` solves a key for them,
// including from inside a generic fn whose `T` has no bound.
struct P(a: Int, b: Int)
fn by_a<T>(xs: List<T>, before: (T, T) -> Bool) -> List<T> = xs.sorted(cmp: before)
var xs = [P(2, 1), P(1, 2)]
xs.sort(cmp: (p, q) => p.a < q.a)
print(xs[0].a, xs[1].a)
let ys = by_a([P(3, 0), P(1, 0)], (p, q) => p.a < q.a)
print(ys[0].a, ys[1].a)
