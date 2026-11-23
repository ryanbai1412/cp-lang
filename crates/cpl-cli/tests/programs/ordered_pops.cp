// The ends of an ordered `Set` and `Map`: read with `first`/`last`, taken out with
// `pop_first`/`pop_last`; each is `none` once nothing is left.
var s = Set<Int>()
var m = Map<Int, Str>()
print(s.first, s.pop_first() ?? -1, s.pop_last() ?? -1, m.first, m.last, m.pop_first(), m.pop_last())
for x in [5, 1, 9, 3]:
    s.insert(x)
    m[x] = "v{x}"
print(s.first!, s.last!, m.first!, m.last!)
print(s.pop_first()!, s.pop_last()!, s.len, s)
let (k, v) = m.pop_first()!
print(k, v, m.pop_last()!, m.len)
while s.pop_first() != None: m.pop_last()
print(s.len, m.len, yes(s.first == None), yes(m.last == None))
domain P = 0..70
var b = Bitset<P>()
b.add(P(3))
b.add(P(69))
print(b.count, b.len)
