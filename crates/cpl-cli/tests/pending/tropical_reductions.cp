// `max` and `min` over optionals skip `none`, as `max=` does, and are
// `none` when nothing is left.
let n = 4
domain I = 0..n
var best: [I]?Int = none
best[I(1)] = 5
best[I(3)] = 2
print(max(best) ?? 0, min(best) ?? 0)
let xs: List<?Int> = [none, 3, none, 7]
print(max(xs), min(xs))
let none_at_all: List<?Int> = [none, none]
print(max(none_at_all), min(x for x in xs))
let a: ?Int = none
let b: ?Int = 4
print(max(a, b), min(a, b), max(a, a), max(b, 9))
