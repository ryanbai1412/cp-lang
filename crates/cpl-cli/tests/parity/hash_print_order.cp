var h: HashMap<Int, Int> = HashMap()
for i in 0..12: h[i * 7919 % 101] = i
var s: HashSet<Int> = HashSet()
for i in 0..12: s.add(i * 31 % 97)
var tables: List<HashMap<Int, Int>> = [h, HashMap()]
print(h.keys.sorted())
print([(k, h[k]) for k in h.keys.sorted()], sep: " ")
print(s.sorted())
print("{s.sorted()} | {h.values.sorted()}")
print([t.len for t in tables])
h.remove(0)
print(h.keys.sorted())
