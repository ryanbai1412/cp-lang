// expect: E0606
// rule: 07.9.6
// A list's `first` and `last` may give nothing, so they are `?T` and need `!` or `??` to print (bug-catch round 12, 974fbd33).
var xs: List<Int> = []
xs.push(4)
print(xs.first) // <- here
