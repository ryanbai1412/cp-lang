var xs: List<Int> = []
xs.reserve(5)
xs.reserve(0)
assert xs.is_empty
for i in 0..8:
    xs.push(i * i)
xs.reserve(3)
assert xs.len == 8
var nested: List<List<Int>> = [[], []]
nested[0].reserve(4)
nested[0].push(42)
nested[1].reserve(2)
nested[1].push(99)
print(xs.sum(), nested[0][0], nested[1][0])
