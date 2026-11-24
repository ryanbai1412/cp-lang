// expect: E0309
// `sort` takes a key (`by:`) or a comparator (`cmp:`), not both (decision 87).
var a = [2, 1, 3]
a.sort(by: x => -x, cmp: (x, y) => x < y) // <- here
print(a)
