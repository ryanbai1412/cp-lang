var s: Set<Int> = Set()
for x in [3, 7, 10]: s.add(x)
print(s.first_ge(7) ?? -1, s.first_gt(7) ?? -1, s.last_le(7) ?? -1, s.last_lt(7) ?? -1)
print(s.first_ge(11) ?? -1, s.last_lt(3) ?? -1, s.first_ge(-5) ?? -1)
var m: Map<Int, Str> = Map()
m[1] = "one"
m[5] = "five"
print(m.first_ge(2), m.last_le(4), m.first_gt(1), m.last_lt(5))
if m.first_ge(6) == None: print("nothing at or after 6")
let (k, v) = m.first_ge(0) ?? (0, "")
print(k, v)
