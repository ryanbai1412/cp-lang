// expect: E0401 at 2
// rule: 05.6.3
let a = [5, 6]
a.at(1) = 0
print(a[1])
