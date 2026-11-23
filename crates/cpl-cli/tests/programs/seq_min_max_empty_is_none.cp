var xs: List<Int> = []
print(xs.min() ?? -1, xs.max() ?? -1)
xs.push(4)
xs.push(2)
print(xs.min()!, xs.max()!, [3, 1].min()!)
let r = 0..0
print(r.min() ?? 9)
