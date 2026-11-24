// expect: E0902
// rule: 04.2.18
domain D = 0..3
fn right(b: Cut<D>) -> Int: Int(b)
print(right(5)) // <- here
