// expect: ok, output "2 0 1\n30 20 10"
// rule: std.seq sorted
// A domain is no place to sort in: `D.sorted(by:)` gives a new `List`, and
// `sort` sorts a list in place (decision 627).
domain I = 0..3
let w: [I]Int = [I] i => [5, 9, 1][i.ord]
let order = I.sorted(by: i => w[i])
print(order.map(i => i.ord).to_list())
var xs = [10, 30, 20]
xs.sort(by: x => -x)
print(xs)
