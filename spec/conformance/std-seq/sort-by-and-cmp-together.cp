// expect: E0310
// `sort` takes a key (`by:`) or a comparator (`cmp:`), not both (decision 87): they are two
// overloads, and a call fitting none is the named-argument error (decisions.md, R06.1.6).
var a = [2, 1, 3]
a.sort(by: x => -x, cmp: (x, y) => x < y) // <- here
print(a)
