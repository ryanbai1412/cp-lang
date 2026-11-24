// expect: ok, output "0 3\n2 4\n4 5"
let a = [3, 1, 4, 1, 5]
for (k, x) in a.filter(x => x > 1).items: print(k, x)
