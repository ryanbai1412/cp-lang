// expect: ok, output "1 2 -1"
// rule: 07.12.9
var m: Map<Int, Int> = [:]
m[5] = 1
m[6] += 2
print(m[5], m[6], m.get(7) ?? -1)
