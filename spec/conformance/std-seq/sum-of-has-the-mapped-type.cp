// expect: ok, output "5 6"
// rule: std.seq.sum
// `sum(of:)` has the type `of:` maps to, not the element's: a list of domain positions sums
// to an Int.
domain T = [3]
fn price(xs: List<T>) -> Int = xs.sum(of: x => x.ord + 2)
let a: [T]Int = [T] t => t.ord
print(price([T.first!, T.last!]) - 1, a.sum(of: x => x + 1))
