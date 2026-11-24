var h: HashMap<Int, Int> = HashMap()
h[5] = 1
h[2] = 7
var s: HashSet<Int> = HashSet()
s.add(3)
s.add(1)
print(s.sorted())
print("{s.sorted()}")
print(h.keys.sorted())
print(h.values.sorted())
print("{h.keys.sorted()} {h.values.sorted()}")
