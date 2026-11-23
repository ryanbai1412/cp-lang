domain V = 1..=6
let a = [3, 1, 4, 1, 5, 9, 2]
var s = 0
for x in a.step(3): s = s * 10 + x
print(s)
print([x for x in a.step(2)])
print([Int(v) for v in (V(2)..=V(6)).step(2)])
let e = a.step(4)
var t = 0
for x in e: t += x
print(t, e.len)
print([c for c in "abcde".step(2)])
