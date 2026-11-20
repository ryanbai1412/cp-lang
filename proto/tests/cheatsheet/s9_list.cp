var xs: List<Int> = []
xs.push(3); xs.push(1)
let ys = xs + [9]
print(xs.len, xs.is_empty, xs.pop(), ys.len)
xs.clear()
print(xs.pop(), xs.len)
