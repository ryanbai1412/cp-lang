// expect: ok, output "2\n1 3\n3\n0"
// rule: std.subset.Subset.of
// `Subset<B>.of` takes the members any `Seq` yields, so a set read from input or a filtered
// range becomes a `Subset` without a `var` and a `.with` loop; a repeated member counts once.
domain W = 0..5
let u = Subset<W>.of([W(1), W(3), W(1)])
print(u.len)
print(u.items.collect())
print(Subset<W>.of(W.filter(w => w.ord % 2 == 0)).len)
let none: List<W> = []
print(Subset<W>.of(none).len)
