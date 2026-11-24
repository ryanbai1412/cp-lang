// expect: E0606
// rule: 10.3.8
// Interpolation applies `print`'s Show check (R10.3.2), so a `HashSet` in a string is E0606 too.
var h: HashSet<Int> = HashSet()
h.add(1)
print("{h}") // <- here
