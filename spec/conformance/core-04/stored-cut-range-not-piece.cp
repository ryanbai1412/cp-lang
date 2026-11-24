// expect: E0302
// rule: 04.4.5 04.8.5
domain V = 1..=5
let a = [V] v => Int(v)
fn cost(r: Range<V>) -> Int = a[r].sum()
let s = V.cuts(1)..V.cuts(3)
print(cost(s)) // <- here
