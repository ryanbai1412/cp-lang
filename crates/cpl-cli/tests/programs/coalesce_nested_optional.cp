// `??` on an optional of optionals: `xs.last` of a `List<?Int>` is `none`
// for an empty list and the last cell, itself optional, otherwise. A plain
// default opens one level; an optional default keeps the optional.
fn fmt_i(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
var cs: List<?Int> = [none, 5]
let x = cs.last ?? 7
print(fmt_i(cs.last ?? 7), yes(x == 5), yes(x == none), fmt_i(cs.first ?? 7), yes((cs.first ?? 7) == none))
var empty: List<?Int> = []
let y = empty.last ?? 7
print(y ?? -1, yes(y == 7), yes((empty.first ?? none) == none))
let a: ?Int = none
let b: ?Int = 3
print(a ?? b ?? 0, yes((a ?? b) == 3), a ?? 9, b ?? 9)
var ps: List<?Bool> = [true, none]
print(yes((ps.first ?? false) ?? false), yes((ps.last ?? true) == none))
