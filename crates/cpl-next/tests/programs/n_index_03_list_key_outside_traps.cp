// expect: R0101
// rule: R07.2.17
let xs = [1, 2, 3]
var i = 2
print(xs[i])
i += 1
print(xs[i]) // <- here
