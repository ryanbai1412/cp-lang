// expect: ok, output "0\n1\n2\n3\n4"
let a = [3, 1, 4, 1, 5]
for k in a.map(x => x * 2).keys: print(k)
