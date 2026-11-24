// expect: ok, output "9\n12\n8"
// rule: std.subset.Subset
// `D.subsets` over a generic domain `D` is `Subset<D>`, the struct std declares
// `@member(subsets)`: its members are `Subset`'s methods, and `List(B.subsets)`
// collects every subset the way `List(B)` collects every position.
domain B = 0..3
fn best<D>(d: D) -> Int:
    var m = 0
    for s in D.subsets:
        if s.contains(d): m = max(m, s.without(d).len + Int(s))
    m
print(best(B(1)))
let ys = List(B.subsets)
print(ys.map(s => s.len).sum())
print(ys.len)
