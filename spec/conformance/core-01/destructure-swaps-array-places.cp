// expect: ok, output "3 2 1"
// rule: 01.6.4
var p = [1, 2, 3]
let l = 0
let r = 2
(p[l], p[r]) = (p[r], p[l])
print(p)
