// expect: ok, output "YES YES 0..3 3..6 2..3 3..5\n2 3\nYES\n\n1\n1 2\n1 2 3\n1 3\n2\n2 3\n3\n0\n1\n1\nYES\n1\n7"
use std.bag.{SortedBag, SortedSet, SortedMap}
use std.hashmap.{Counter}
use std.window.{track}
print((0..5).halve() == Some((0..2, 2..5)), (3..4).halve() == None, (0..6).halve(), (2..=4).halve())
domain D = 0..5
let (a, b) = D.all.halve()!
print(a.len, b.len)
print([1, 2].subsequences().map(s => s.items).collect() == [[], [1], [1, 2], [2]])
print([s.items for s in [1, 2, 3].subsequences()])
print([s.items.len for s in "".subsequences()])
var bg = SortedBag.of([1]); bg.reserve(100); print(bg.to_list())
var st = SortedSet.of([1]); st.reserve(100); print(st.to_list())
var m: SortedMap<Int, Str> = SortedMap(); m.reserve(100); print(m.is_empty)
var c = Counter.of([1]); c.reserve(100); print(c[1])
print(track(7, (mut s, x) => s += x, (mut s, x) => s -= x).state)
