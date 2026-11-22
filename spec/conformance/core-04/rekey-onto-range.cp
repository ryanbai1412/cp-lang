// expect: ok, output "2"
// rule: 04b.1.7 04b.4.5 04b.5.9
let k = [1, 2, 3].rekey(5..8)
let i = 6
print(k[i])
