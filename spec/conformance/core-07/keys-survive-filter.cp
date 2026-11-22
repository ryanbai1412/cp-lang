// expect: ok, output "0\n2"
// rule: 07.3.2
let a = [5, 6, 7]
for (i, x) in a.filter(x => x != 6).items: print(i)
