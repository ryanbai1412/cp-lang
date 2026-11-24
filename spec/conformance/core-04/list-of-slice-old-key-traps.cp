// expect: R0101
// rule: 04a.11.6
let a = [10, 20, 30, 40]
let c = List(a[2..])
print(c[2]) // <- here
