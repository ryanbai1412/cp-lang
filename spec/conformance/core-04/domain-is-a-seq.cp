// expect: ok, output "2 2 2 1"
// rule: 04.2.20
domain V = 1..=4
let a: [V]Int = [V] v => Int(v) * Int(v) % 5
print(V.find(v => a[v] == 4)!, V.count(v => a[v] == 4), V.filter(v => a[v] == 1).count(), V.min(by: v => a[v])!)
