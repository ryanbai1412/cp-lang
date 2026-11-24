// here: 4
// C++'s `a.insert(a.begin() + i, x)` (self-run mistakes): "no method `begin`", no fix.
var a: List<Int> = [3, 1, 2]
a.insert(a.begin() + 1, 7)
