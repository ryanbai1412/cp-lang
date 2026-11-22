// warning L0601: `xs` is changed on the right side of an assignment to `xs[0]`
var xs: List<Int> = [5, 6, 7]
xs[0] = xs.pop()!
print(xs)
