// expect: ok, output "3\n5\n2"
// rule: 04.4.20 04.8.3
domain V = 1..=5
let a = [V] v => Int(v) * 2
print(Int(a.keys.bsearch(i => a[i] >= 7)))
print(Int(a.keys.bsearch(i => a[i] >= 99)))
print(Int(a.keys.bsearch(i => a[i] <= 4, last: true)))
