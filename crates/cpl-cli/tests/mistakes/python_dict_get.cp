// here: 4
// Python's `d.get(k, 0)` (self-run mistakes): a default is `?? 0`.
var m = Map<Str, Int>()
print(m.get("a", 0) + 1)
