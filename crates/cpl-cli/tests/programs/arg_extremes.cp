// `argmax(f(v) for v in V)` is the `v` with the largest `f(v)`: the first on a
// tie, `none` when nothing is left, and a `none` among the values is skipped.
let best: List<?Int> = [None, 4, None, 9, 1]
print(argmax(best[i] for i in 0..best.len), argmin(best[i] for i in 0..best.len))
domain I = 0..3
var t: [I]?Int = None
print(argmax(t[i] for i in I))
t[I(1)] = 7
print(argmax(t[i] for i in I), argmin(i.ord for i in I if t[i] == None))
let xs = [3, 9, 2, 9]
print(argmax(xs[i] for i in 0..4), argmin(xs[i] for i in 0..4))
