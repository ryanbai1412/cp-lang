// expect: ok, output "2 bc"
// A sorted container's `range(r: Range<K>)` declares the view's key: here `Str` for a `SortedSet<Str>`.
use std.bag.{SortedSet}
var s = SortedSet<Str>()
for w in ["a", "b", "c", "d"]: s.add(w)
var o = ""
for w in s.range("b".."d"): o += w
print(s.range("b".."d").len, o)
