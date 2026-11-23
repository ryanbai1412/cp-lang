// 1954C Long Multiplication: equal sums, so make the two numbers as close as possible.
@!multitest
var x = read<Str>()
var y = read<Str>()
var split = false
for i in 0..x.len:
    let hi = max(x[i], y[i])
    let lo = min(x[i], y[i])
    x[i] = if split: lo else: hi
    y[i] = if split: hi else: lo
    if hi != lo: split = true
print(x)
print(y)
