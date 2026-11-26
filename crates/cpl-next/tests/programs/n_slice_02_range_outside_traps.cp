// expect: R0101
// rule: R07.4.5
let xs = [5, 6, 7]
var j = 3
print(xs[1..j].len)
j += 1
print(xs[1..j].len) // <- here
