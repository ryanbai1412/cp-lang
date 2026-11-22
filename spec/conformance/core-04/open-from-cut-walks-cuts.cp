// expect: ok, output "9"
// rule: 04.4.10 04.8.6
domain V = 1..=4
var n = 0
for j in V(2).r..: n += Int(j)
print(n)
