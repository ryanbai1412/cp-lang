// expect: ok, output "1 2 9 2"
// engines: next
let a = [1, 2]
var b = a.clone()
b[0] = 9
print(a, b)
