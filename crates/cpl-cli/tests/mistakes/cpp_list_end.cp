// here: 4
// C++'s `a.end() - 1` as a position (self-run mistakes): "no method `end`", no help.
var a: List<Int> = [3, 1, 2]
print(a[a.end() - 1])
