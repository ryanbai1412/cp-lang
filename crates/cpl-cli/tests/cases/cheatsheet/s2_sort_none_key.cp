// cp: 0.1 bea787c+dirty
var xs: List<?Int> = [3, None, 1]
sort(mut xs, by: x => (x == None, x ?? 0))
print(xs)
