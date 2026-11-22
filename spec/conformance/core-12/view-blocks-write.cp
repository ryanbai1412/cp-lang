// expect: E0602 at 3
// rule: 12.4.9 12.4.11
var xs = [1, 2, 3]
let v = xs.filter(x => x > 1)
xs.push(4)
print(v.sum())
