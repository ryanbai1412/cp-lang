// expect: E0602
// rule: 05.7.4 05.9.1
// Every lazy adapter over `a` (filter, map, take, adjacent, with_prev, cyclic_pairs) keeps `a` lent for the walk.
var a = [3, 1, 2, 4]
for x in a.filter(x => x > 0).map(x => x + 1):
    a.push(x) // <- here
print(a.len)
