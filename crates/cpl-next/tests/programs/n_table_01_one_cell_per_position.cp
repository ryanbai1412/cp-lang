// expect: ok, output "0 2 4\n1 1 2\n3"
// rule: R02.8.2
// `[D] k => e` makes one cell per position of `D`, in its walk order.
domain V = 0..3
let t: [V]Int = [V] v => Int(v) * 2
print(t)
var calls = 0
let u = [3] i =>
    calls += 1
    if i == 2: 2 else: 1
print(u)
print(calls)
