// `s.set(v, on)` puts `v` in a subset or takes it out, by a Bool.
domain V = 0..5
var s = Subset<V>.empty
s = s.set(V(1), true)
s = s.set(V(3), true)
s = s.set(V(3), true)
print(s.len, V(1) in s, V(3) in s, V(2) in s)
s = s.set(V(1), false)
s = s.set(V(4), false)
print(s.len, V(1) in s, V(3) in s)
for v in V: s = s.set(v, v.ord % 2 == 0)
print([v.ord for v in V if v in s])
