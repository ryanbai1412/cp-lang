// expect: ok, output "3"
domain P = 0..4
fn right<D: Domain>(d: D) -> Cut<D> = d.r
print(right(P.cuts.nth(2)).ord)
