// expect: E0310
// tool: check
// has: "its overloads are `sort(by:)` and `sort(cmp:)`"
// `sort(by:)` and `sort(cmp:)` are two fns of one name, told apart by their labels (R06.1.6), so no `sort` takes both (decisions 87, 624): the error names both and the label to drop.
var a = [2, 1, 3]
a.sort(by: x => -x, cmp: (x, y) => x < y) // <- here
print(a)
