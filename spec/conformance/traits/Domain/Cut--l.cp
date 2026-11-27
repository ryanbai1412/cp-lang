// expect: ok, output "2"
// rule: 04.2.31
domain P = 0..4
fn left<D: Ordered>(d: D) -> Cut<D> = d.l
print(left(P.cuts.nth(2)).ord)
