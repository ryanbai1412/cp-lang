// expect: ok, output "10\n15\n12\n3"
// rule: 04.4.8 04.4.9 04.4.10 04.8.6
domain V = 1..=5
let a = [V] v => Int(v)
print(a[..5].sum())
print(a[..=5].sum())
print(a[3..].sum())
print(a[..V.cuts(2)].sum())
