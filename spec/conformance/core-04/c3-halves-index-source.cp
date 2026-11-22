// expect: ok, output "10\n50"
// rule: 04a.4.3
domain V = 0..4
let a = [V] v => Int(v) * 10
let (x, y) = V.range.halve()!
print(a[x].sum())
print(a[y].sum())
