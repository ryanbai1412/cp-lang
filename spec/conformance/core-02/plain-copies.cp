// expect: ok, output "7"
// rule: 02.1.1
let a = (x: 3, y: 4)
var b = a
b.x = 3
print(a.x + b.y)
