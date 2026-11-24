// `argmax(f(v) for v in V)` is the `v` with the largest `f(v)`: the first on a
// tie, `none` when nothing is left, and a `none` among the values is skipped.
let best: List<?Int> = [None, 4, None, 9, 1]
print(argmax(best[i] for i in 0..best.len), argmin(best[i] for i in 0..best.len), none: "none")
domain I = 0..3
var t: [I]?Int = None
print(argmax(t[i] for i in I), none: "none")
t[I(1)] = 7
print(argmax(t[i] for i in I), argmin(i.ord for i in I if t[i] == None), none: "none")
let xs = [3, 9, 2, 9]
print(argmax(xs[i] for i in 0..4), argmin(xs[i] for i in 0..4), none: "none")
// The position is what the `for`s bind: over several, or over a pattern, the
// tuple of the names they bind, in order.
let m: ?I = argmax(t[i] for i in I)
print(m!.ord)
print(argmax(i * j for i in 0..3 for j in 0..2)!, argmin(i - j for i in 0..3 for j in 0..2)!)
print(argmax(w for (k, w) in [(0, 3), (1, 5), (2, 4)])!)
