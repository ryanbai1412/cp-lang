// `List(s.rev)` is a list of `s`'s elements, so its `sum` is an `Int`
// wherever it is used. Found by `cpl-conform fuzz`, seed 49.
// Rule compiler-changes/R5: a view is not its source, but walks its items.
let a = [1, 2, 3]
var v = 0
v = List(a.rev).sum()
let w: List<Int> = List(a.filter(x => x > 1).rev)
let s: List<Char> = List("ab".rev)
print(v, w, s)
