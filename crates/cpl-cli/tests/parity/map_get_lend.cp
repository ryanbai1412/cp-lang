var m = Map<Int, List<Int>>()
m[0] = [1, 2, 3]
if let l = m.get(0):
    m[0] = []
    print(l.len)
while let w = m.get(0):
    print(w.len)
    m.remove(0)
print(m.len)
var h = HashMap<Int, List<Int>>()
h[1] = [5, 6]
var s = 0
for i in 0..4:
    if let l = h.get(1):
        s += l.len + l[0]
    if let l = h.get(2):
        s += 100 + l.len
print(s)
