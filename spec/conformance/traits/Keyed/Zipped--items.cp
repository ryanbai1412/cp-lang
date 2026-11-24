// expect: ok, output "2 3 7\n3 4 8"
let a = [1, 2, 3, 4]
for (k, (x, y)) in zip(a[2..], [7, 8]).items: print(k, x, y)
