// expect: ok, output "4"
// rule: 04.2.6
domain V = 1..=4
var n = 0
for x in 0..=6:
    if x in V: n += 1
print(n)
