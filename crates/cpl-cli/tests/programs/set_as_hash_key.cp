// A `Set` keys a `Map` or a `Set`: equal sets built in any order are one key, in the C++ as in the interpreter.
domain Q = 0..3
var seen: Map<Set<Q>, Int> = Map()
seen[Set([Q(1)])] = 2
seen[Set([Q(2), Q(1)])] = 3
print(seen.len, seen.get(Set([Q(1), Q(2)])) ?? 0)
var sets: Set<Set<Q>> = Set()
sets.add(Set([Q(0), Q(2)]))
sets.add(Set([Q(2), Q(0)]))
sets.add(Set([Q(1)]))
print(sets.len, if Set([Q(2), Q(0)]) in sets: 1 else: 0)
