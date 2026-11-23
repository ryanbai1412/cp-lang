// expect: ok, output "3 4"
// rule: 07.4.5
// A literal inside a slice's constant keys reads that key (decision 388).
let a = [1, 2, 3, 4, 5]
let w = a[1..4]
print(w[2], a[1..=3][3])
