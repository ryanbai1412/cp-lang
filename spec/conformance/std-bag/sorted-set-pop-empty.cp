// expect: ok, output "-1 -1"
var s: SortedSet<Int> = SortedSet()
print(s.pop_first() ?? -1, s.pop_last() ?? -1)
