// expect: ok, output "0"
// rule: 03.1.9
// input: "5\n"
let n: 1..=2e5 = read()
domain V = 0..n
let a: [V]Int = [V].filled(0)
print(a[V(0)])
