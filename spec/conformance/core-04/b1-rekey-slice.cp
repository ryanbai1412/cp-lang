// expect: ok, output "8 9\n0 1"
// rule: 04b.1.7
let a = [5, 6, 7, 8, 9]
let w = a[3..5].rekey(0..2)
print(w[0], w[1])
print(w.keys.collect())
