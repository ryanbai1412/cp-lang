// here: 4
// C++'s `a.resize(n)` (self-run mistakes): "`resize` takes 2 arguments, but 1 was given", no fix.
var a: List<Int> = [3, 1]
a.resize(4)
print(a)
