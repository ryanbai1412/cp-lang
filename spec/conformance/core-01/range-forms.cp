// expect: ok, output "24"
// rule: 01.8.4
var s = 0
for i in 0..10 by 3: s += i
for i in 1..=3: s += i
print(s)
