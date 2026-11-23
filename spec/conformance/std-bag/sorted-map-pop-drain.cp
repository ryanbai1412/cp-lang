// expect: ok, output "1 3\n4"
var m: SortedMap<Int, Int> = SortedMap()
m.set(3, 30)
m.set(1, 10)
m.set(4, 40)
let (lo, _) = m.pop_first()!
let (hi, _) = m.pop_last()!
print(lo, m.first!.0)
print(hi)
