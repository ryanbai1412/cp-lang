domain D = 0..3
var b: [D]?Int = None
print(b.max() ?? -1)
b[1] = 5
b[2] = 2
let hi = b.max()
let lo = b.min()
print(hi ?? -1, lo ?? -1, b.max()! + 1)
let xs: List<?Int> = [None, 7, None, 3]
print(xs.max()!, xs.min()!, max(x for x in xs) ?? -1)
