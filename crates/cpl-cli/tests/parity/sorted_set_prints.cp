use std.bag.{SortedBag, SortedMap, SortedSet}
struct Held:
    s: SortedSet<Int>
    tag: Int
var s = SortedSet<Int>()
print("[{s}]")
for x in [50, 7, 91, 7, 3, 64, 12, 3]: s.add(x)
print(s)
print("[{s}]")
print(s.len, s)
s.remove(64)
print(s)
let h = Held(s.clone(), 1)
print(h)
print([s.clone(), SortedSet.of([2, 1])])
var w = SortedSet<Str>()
for x in ["pear", "apple", "fig"]: w.add(x)
print(w)
print(SortedBag.of([3, 1, 3, 2]))
var m = SortedMap<Int, Int>()
m[9] = 1
m[4] = 2
print(m)
