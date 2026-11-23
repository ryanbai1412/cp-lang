// expect: ok, output "2 3 0"
// rule: 04.2.18
// `Cut<D>` is how library code spells `D.cuts`; a literal fills either.
domain D = 0..3
fn left(b: D.cuts) -> Int: Int(b)
fn right(b: Cut<D>) -> Int: Int(b)
print(left(2), right(3), right(0))
