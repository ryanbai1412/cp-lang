// expect: ok, output "15"
// rule: 13.3.2
var k = 1
let add = (x: Int) => x + k
k = 10
print(add(5))
