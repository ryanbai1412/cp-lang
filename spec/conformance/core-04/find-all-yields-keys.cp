// expect: ok, output "3"
// rule: 04b.5.2
let s = "abcab"
var t = 0
for k in s[1..].find_all("ab"): t += k
print(t)
