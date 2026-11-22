// expect: E0305 at 3
// rule: 04a.9.5
let a = [10, 20, 30]
let f = a.filter(x => x > 10)
print(f[1])
