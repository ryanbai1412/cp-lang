// expect: ok, output "2"
// engines: next
var h: HashSet<Int> = HashSet()
for x in [4, 1, 4]: h.add(x)
print(h.len)
