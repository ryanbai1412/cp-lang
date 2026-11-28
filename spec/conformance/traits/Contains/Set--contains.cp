// expect: ok, output "1 0"
// engines: next
var s = Set<Int>()
s.add(3)
s.add(1)
print(if s.contains(3): 1 else: 0, if s.contains(2): 1 else: 0)
