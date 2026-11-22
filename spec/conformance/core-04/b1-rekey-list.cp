// expect: ok, output "5\n11 5\n12 6"
// rule: 04b.1.7
let w = [4, 5, 6].rekey(10..13)
print(w[11])
for (k, x) in w.items.drop(1): print(k, x)
