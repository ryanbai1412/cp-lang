// expect: ok, output "3 12"
// rule: R04.4.8
// A slice with one end left out and the other a cut is the piece of positions before or after it.
domain V = 1..=5
let a = [V] v => Int(v)
let c = V.cuts(2)
print(a[..c].sum(), a[c..].sum())
