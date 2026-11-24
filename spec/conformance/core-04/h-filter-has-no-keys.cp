// expect: E0305
// rule: 04a.9.5
let a = [10, 20, 30]
let f = a.filter(x => x > 10)
print(f[1]) // <- here
