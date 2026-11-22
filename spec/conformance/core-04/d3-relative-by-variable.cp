// expect: ok, output "4\n6"
// rule: 04a.5.3 04b.1.4
let s = [1, 2, 3, 4, 5, 6]
let k = 3
print(s[2..][k])
print(s[2..].nth(k))
