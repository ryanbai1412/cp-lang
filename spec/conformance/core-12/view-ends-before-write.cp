// expect: ok, output "5 4"
// rule: 12.4.9
var xs = [1, 2, 3]
let v = xs.filter(x => x > 1)
let s = v.sum()
xs.push(4)
print(s, xs.len)
