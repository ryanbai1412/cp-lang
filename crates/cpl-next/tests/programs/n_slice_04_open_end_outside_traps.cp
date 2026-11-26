// expect: R0101
// rule: R07.4.6
let xs = [5, 6, 7]
var k = 3
print(xs[k..].len)
k += 1
print(xs[k..].len) // <- here
