// expect: ok, output "0 6\n1 2\n2 8\n3 2\n4 10"
let a = [3, 1, 4, 1, 5]
for (k, x) in a.map(x => x * 2).items: print(k, x)
