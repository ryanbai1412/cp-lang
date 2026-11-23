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
fn total(m: Map<Int, List<Int>>, fallback: List<Int>) -> Int:
    var t = 0
    for i in 0..3:
        let l = m.get(i) ?? fallback
        let k = m.get(i + 1) ?? [i, i]
        t += l.len * 10 + l[0] + k.len
    t
var c = Map<Int, List<Int>>()
c[0] = [4, 5]
c[2] = [6]
print(total(c, [9]))
let before = c.get(0) ?? []
c[0] = []
var grown = c.get(7) ?? [1]
grown.push(2)
print(before.len, c[0].len, grown.len)
var w = Map<Int, List<Int>>()
w[0] = [1, 2, 3]
var t = 0
for k in 0..2:
    for v in w.get(k) ?? [7]: t += v
for v in w.get(0) ?? []: w[0] = [v * 10]
print(t, w[0])
