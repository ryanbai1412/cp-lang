// here: 4
// Python's `key=` for a sort key (self-run mistakes): CP Lang names it `by:`, and the fix renames it.
var a = [3, 1, 2]
a.sort(key: x => -x)
print(a)
