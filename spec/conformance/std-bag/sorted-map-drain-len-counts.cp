// expect: ok, output "2 2\n0"
var m: SortedMap<Int, Int> = SortedMap()
for k in [1, 3, 5, 7]: m.set(k, k * 10)
let gone = m.drain(after: 2, while: (k, _) => k <= 5).len
print(gone, m.len)
for _ in m.drain(while: (_, _) => true): break
print(m.len)
