// expect: ok, output "3"
// rule: 04.2.29
// R04.2.29: a domain over a list has one position per element.
domain V = [4, 5, 6]
var s = 0
for v in V: s += 1
print(s)
