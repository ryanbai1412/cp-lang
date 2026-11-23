let n: 1..=1000 = read()
var tri: List<List<Int>> = [[0 for _ in 0..=i] for i in 0..n]
tri[n - 1][0] = 7
print(tri[n - 1][0])
