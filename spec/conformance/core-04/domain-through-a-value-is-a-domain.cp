// expect: ok, output "3\n1\n1\n0\n2"
// rule: 04.2.20
// A domain reached through a value (`vt.K`) is a domain like a named one: `.all`, `.find`, `.first`.
domain V = 0..5
let t = Rooted([V] v => if v.ord == 0: [V(1), V(2), V(3), V(4)] else: [V(0)], V(0))
let vt = t.virtual([V(1), V(2)])
print(vt.K.all.len)
print(vt.K.find(k => vt.at[k] == V(1))!.ord)
print(vt.K.all.find(k => vt.at[k] == V(1))!.ord)
print(vt.K.first!.ord)
print(vt.K.last!.ord)
