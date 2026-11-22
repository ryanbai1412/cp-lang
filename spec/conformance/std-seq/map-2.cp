// expect: ok, output "YES"
domain V = 1..=3
let a: [V]Int = [4, 5, 6]
print(a.map(x => -x)[V(2)] == -5)
