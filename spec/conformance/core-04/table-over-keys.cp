// expect: ok, output "9\n0"
// rule: 04b.1.5 04b.4.2
let s = [4, 5, 6, 7]
let w = s[1..3]
var t = [w.keys]0
t[2] = 9
print(t[2])
print(t[1])
