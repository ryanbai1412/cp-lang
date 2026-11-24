// expect: ok, output "3 2 1\n30 5 YES\n2 1"
// rule: 07.11.1
// A map's order is its key type's `Ord`, a user one too, on both engines.
var m = Map<Desc<Int>, Int>()
m[desc(1)] = 10
m[desc(3)] = 30
m[desc(2)] += 5
print([k.x for k in m.keys])
print(m[desc(3)], m[desc(2)], yes(m.get(desc(9)) == None))
m.remove(desc(3))
print([k.x for k in m.keys])
