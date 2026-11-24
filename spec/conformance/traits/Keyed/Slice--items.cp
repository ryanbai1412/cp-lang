// expect: ok, output "2 4\n3 1\n4 5"
let a = [3, 1, 4, 1, 5]
for (k, x) in a[2..].items: print(k, x)
