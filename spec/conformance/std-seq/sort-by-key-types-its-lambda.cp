// expect: ok, output "1 3 1 2 3"
// A List or array member that takes std's row keeps its types: the lambda `sort(by:)` or
// `sort(cmp:)` is given reads its element, and the call is a `()`.
var hills: [3](u: Int, d: Int) = [(u: 1, d: 3), (u: 2, d: 1), (u: 3, d: 2)]
hills.sort(by: h => h.d)
var xs = [3, 1, 2]
xs.sort(by: x => -x)
let ys = xs.sorted(by: x => x)
var zs = [2, 3, 1]
zs.sort(cmp: (a, b) => a > b)
print(hills[0].d, xs[0], ys[0], xs.take(2).len, zs[0])
