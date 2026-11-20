// `??` on an optional of optionals: `xs.last` of a `List<?Int>` is `none`
// for an empty list and the last cell, itself optional, otherwise. A plain
// default opens one level; an optional default keeps the optional.
var cs: List<?Int> = [none, 5]
let x = cs.last ?? 7
print(cs.last ?? 7, x == 5, x == none, cs.first ?? 7, (cs.first ?? 7) == none)
var empty: List<?Int> = []
let y = empty.last ?? 7
print(y ?? -1, y == 7, (empty.first ?? none) == none)
let a: ?Int = none
let b: ?Int = 3
print(a ?? b ?? 0, (a ?? b) == 3, a ?? 9, b ?? 9)
var ps: List<?Bool> = [true, none]
print(ps.first ?? false, (ps.last ?? true) == none)
