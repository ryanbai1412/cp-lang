// expect: ok, output "0 1 0 1 2\n5 1 0 2 1\n0 0 0 0 0"
// rule: 04b.5.3
domain V = 1..=5
let arr: [V]Int = [1, 1, 2, 1, 1]
print(prefix_function(arr))
print(z_function(arr))
print(prefix_function(1..=5))
