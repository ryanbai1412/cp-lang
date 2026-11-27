// expect: ok, output "1\n3"
var s = Set<Int>()
s.insert(3)
s.insert(1)
for x in s: print(x)
