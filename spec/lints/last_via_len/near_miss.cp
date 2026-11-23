let n: 1..=100 = read()
var xs: List<Int> = [n]
xs.push(n * 2)
xs[xs.len - 1] = 0
print(xs[xs.len - 2])
