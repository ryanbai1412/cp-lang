// expect: E0401
// rule: 05.6.3
let a = [5, 6]
a.at(1) = 0 // <- here
print(a[1])
