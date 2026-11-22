// expect: ok, output "5 6"
// rule: 05.2.1
let p = (5, [1, 2].len)
var q = p
q.0 += 1
print(p.0, q.0)
