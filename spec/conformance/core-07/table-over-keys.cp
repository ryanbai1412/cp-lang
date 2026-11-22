// expect: ok, output "9 2"
// rule: 07.4.3
let s = [5, 6, 7, 8]
let w = s[1..3]
var t: [w.keys]Int = 0
t[2] = 9
print(t[2], t.len)
