// expect: ok, output "5"
// engines: next
var h: HashSet<Int> = HashSet()
for x in [4, 1, 4]: h.add(x)
var s = 0
for x in h: s += x
print(s)
