// expect: E0310
// tool: check
// has: "drop `cmp:` to call `sort(by:)`, or drop `by:` to call `sort(cmp:)`"
// The free form fits no `sort` either, and says which label to drop (decision 624).
var a = [2, 1, 3]
sort(mut a, by: x => -x, cmp: (x, y) => x < y) // <- here
print(a)
