// expect: L0602 at 4
// rule: 07.1.3 07.2.9
let a = [1, 2, 3]
let c = a.filter(x => x > 1).map(x => x * 2)
print(c.sum())
print(c.sum())
