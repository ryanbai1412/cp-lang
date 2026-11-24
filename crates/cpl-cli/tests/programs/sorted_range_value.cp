// A range kept as a value sorts to what it goes over: the keys of a table or a list, a domain, a range bound to a name, in both engines.
domain R = 0..3
var a: [R]Int = 0
print([Int(k) for k in a.keys.sorted()])
print(a.keys.sorted().len, R.sorted().len)
let v = [5, 1, 3]
print(v.keys.sorted())
print(v.keys.sorted(by: i => desc(i)))
let r = 2..6
print(r.sorted(by: i => desc(i)))
