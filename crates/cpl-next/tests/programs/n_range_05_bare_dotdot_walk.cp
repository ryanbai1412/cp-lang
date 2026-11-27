// expect: E0102
// `for x in ..` names nothing to walk: a bare `..` only goes inside `[...]` (R04.4.11).
var n = 0
for x in ..: n += 1 // <- here
print(n)
