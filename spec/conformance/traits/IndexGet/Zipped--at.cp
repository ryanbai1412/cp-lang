// expect: ok, output "3 8"
// engines: next
let a = [1, 2, 3, 4]
let z = zip(a[1..], [7, 8, 9])
print(z[2])
