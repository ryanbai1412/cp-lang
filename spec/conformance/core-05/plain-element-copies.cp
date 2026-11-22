// expect: ok, output "1 5"
// rule: 05.3.5
var a = [1, 2]
let x = a[0]
a[0] = 5
print(x, a[0])
