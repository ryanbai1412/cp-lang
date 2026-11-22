// expect: ok, output "3"
// rule: 12.4.11
var xs = [1, 2]
let f = () => xs.len
xs.push(3)
print(f())
