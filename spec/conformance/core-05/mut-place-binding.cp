// expect: ok, output "12"
// rule: 05.3.3 05.3.4
var a = [1, 2, 3]
let p = mut a[1]
p += 10
print(a[1])
