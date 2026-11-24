// expect: ok, output "3 7\n3 b\n0"
// `Counter.reserve` and `SortedMap.reserve` reserve their hash table too and keep every entry.
var c = Counter.of([1, 1, 2, 3, 1])
c.reserve(1000)
c.add(2, 6)
print(c[1], c[2])
var m: SortedMap<Int, Str> = SortedMap()
m[2] = "b"
m[1] = "a"
m[3] = "c"
m.reserve(1000)
print(m.len, m[2])
var e: SortedMap<Int, Int> = SortedMap()
e.reserve(0)
print(e.len)
