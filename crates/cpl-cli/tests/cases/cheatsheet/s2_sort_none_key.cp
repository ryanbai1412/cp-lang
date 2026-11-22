// cp: 0.1 bea787c+dirty
var xs: List<?Int> = [3, none, 1]
sort(mut xs, by: x => (x == none, x ?? 0))
print(xs)
