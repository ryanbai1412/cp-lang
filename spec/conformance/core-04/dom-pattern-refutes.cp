// expect: ok, output "3"
// rule: 04.2.14 04.8.9
domain V = 1..=5
var s = 0
for i in 0..=12:
    if let V(j) = i - 10: s += Int(j)
print(s)
