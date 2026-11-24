// expect: ok, output "2 2 7 2 YES"
// rule: 07.12.9
// A `Set` keying a `Map` or a `Set` is equal by its members, whatever order they were added in, and the empty set is a key of its own.
var m: Map<Set<Int>, Int> = Map()
var a: Set<Int> = Set()
for x in [3, 1, 2, 100, -5]:
    a.add(x)
var b: Set<Int> = Set()
for x in [-5, 100, 2, 1, 3]:
    b.add(x)
m[a] = 1
m[b] = m[b] + 1
var empty: Set<Int> = Set()
m[empty] = 7
var seen: Set<Set<Int>> = Set()
seen.add(a)
seen.add(b)
seen.add(empty)
print(m.len, m[a], m[empty], seen.len, yes(b in seen))
