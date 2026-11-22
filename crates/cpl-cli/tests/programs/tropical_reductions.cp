// `max` and `min` over optionals skip `none`, as `max=` does, and are
// `none` when nothing is left.
let n = 4
domain I = 0..n
var best: [I]?Int = none
best[I(1)] = 5
best[I(3)] = 2
fn fmt_i(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
print(best.max() ?? 0, best.min() ?? 0)
let xs: List<?Int> = [none, 3, none, 7]
print(fmt_i(xs.max()), fmt_i(xs.min()))
let none_at_all: List<?Int> = [none, none]
print(fmt_i(none_at_all.max()), fmt_i([x for x in xs].min()))
let a: ?Int = none
let b: ?Int = 4
print(fmt_i(max(a, b)), fmt_i(min(a, b)), fmt_i(max(a, a)), max(b, 9))
