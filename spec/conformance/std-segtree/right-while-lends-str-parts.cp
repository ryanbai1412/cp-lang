// expect: ok, output "2"
// A SegTree of strings walked with `right_while`: its seen and blank values are parts of the
// tree, lent to the walk and never copied out.
domain D = 0..4
let st = SegTree<D, Str>(Monoid<Str>("", (a, b) => a + b), [D] i => ["a", "b", "c", "d"][i.ord])
print(st.right_while(D.cuts.nth(0), s => s.len < 3).ord)
