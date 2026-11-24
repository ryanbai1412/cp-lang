// expect: ok, output "1\n2"
let a = [1, 2, 3, 4]
for k in zip(a[1..], [7, 8]).keys: print(k)
