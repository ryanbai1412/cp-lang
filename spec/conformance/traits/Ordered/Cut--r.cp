// expect: ok, output "3"
// engines: next
domain P = 0..4
fn right<D: Ordered>(d: D) -> Cut<D> = d.r
print(right(P.cuts.nth(2)).ord)
