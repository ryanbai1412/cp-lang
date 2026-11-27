// expect: ok, output "2"
var s = Set<Int>()
s.insert(3)
s.insert(1)
s.insert(3)
print(s.len)
