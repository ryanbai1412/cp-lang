// expect: ok, output "0 5\n1 1\n1 cba\n3 1 3 3 0 1"
// rule: 07.2.11
// A closure passed to a built-in member takes what its row's parameter takes, also past `;` (named only) and inside `?(fn)`: `by:` and `cmp:` of `sort` and `sorted` on a `List`, a `Str` and a `[D]T` take the element.
// copies: 1
var xs = [(1, 2), (0, 5), (1, 1)]
xs.sort(cmp: (p, q) => p.0 < q.0)
print(xs[0].0, xs[0].1)
xs.sort(by: p => p.1)
print(xs[0].0, xs[0].1)
let ys = xs.sorted(cmp: (p, q) => p.1 > q.1)
let zs = xs.sorted(by: p => -p.0)
var s = "cab"
s.sort(cmp: (a, b) => a > b)
print(zs[0].0 - ys[0].0 + ys[2].0 - 1, s)
var a = [0..3] i => 3 - i
a.sort(cmp: (x, y) => x > y)
let b = a.sorted(by: x => -x)
let c = "cab".sorted(by: ch => ch.ord)
print(a[0], a[2], b[0], c.len, ys[0].0, zs[0].0)
