// expect: ok, output "15"
// rule: 06.1.4
// A closure lends its captures for each call, so it reads `k` as it is when called; interpreter and C++ agree.
var k = 2
let f = (x: Int) => x * k
k = 5
print(f(3))
