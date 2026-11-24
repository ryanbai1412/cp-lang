// expect: ok, output "4 8\nNO\n8\n6\n4\n2 3\n5 6"
// rule: std.bag.SortedRange
// `Set.range` and `Map.range` return the std `SortedRange` without a `use`:
// the type names it, and its std getters answer the queries.
var st = Set<Int>()
for x in [1, 4, 6, 8, 9]: st.add(x)
let r: SortedRange<Int> = st.range(4..9)
print(r.first ?? -1, r.last ?? -1)
print(yes(r.is_empty))
for x in r.rev: print(x)
var m = Map<Int, Int>()
m[2] = 3
m[5] = 6
for (k, v) in m.range(1..9): print(k, v)
