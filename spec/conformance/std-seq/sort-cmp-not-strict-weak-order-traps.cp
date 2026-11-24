// expect: R0001 (checked)
// A `cmp:` with `cmp(x, x)` true is not a strict weak order (decision 87).
var a = [2, 1, 3]
a.sort(cmp: (x, y) => x <= y) // <- here
print(a)
