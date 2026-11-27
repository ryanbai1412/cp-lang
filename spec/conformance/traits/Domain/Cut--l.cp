// expect: ok, output "2"
domain P = 0..4
fn left<D: Domain>(d: D) -> Cut<D> = d.l
print(left(P.cuts.nth(2)).ord)
