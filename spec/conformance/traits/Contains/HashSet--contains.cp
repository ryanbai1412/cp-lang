// expect: ok, output "YES NO"
// engines: next
var h: HashSet<Int> = HashSet()
h.add(4)
print(yes(h.contains(4)), yes(h.contains(1)))
