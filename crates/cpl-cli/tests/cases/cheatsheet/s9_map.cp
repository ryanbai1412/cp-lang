// cp: 0.1 bea787c+dirty
var m = Map<Int, Int>()
m[1] += 5
m[2] = 7
print(m[1], m.get(3), 2 in m, m.len)
m.remove(1)
for (k, v) in m.items: print(k, v)
