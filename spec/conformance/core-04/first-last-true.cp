// expect: ok, output "7\n8\n-1\n3"
// rule: 04.4.19 04.8.3
let k = (0..=10).last_true(k => k * k <= 50) ?? -1
let m = (0..=10).first_true(k => k * k >= 50)!
print(k)
print(m)
print((0..=10).first_true(k => k > 20) ?? -1)
domain V = 1..=5
let a = [V] v => Int(v) * 2
print(Int(V.range.first_true(v => a[v] >= 5)!))
