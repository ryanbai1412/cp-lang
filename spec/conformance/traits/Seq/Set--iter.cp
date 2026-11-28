// expect: ok, output "1\n3"
// engines: next
var s = Set<Int>()
s.insert(3)
s.insert(1)
for x in s: print(x)
