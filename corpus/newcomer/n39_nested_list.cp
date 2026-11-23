let n: 1..=1000 = read()
var g: List<List<Int>> = [[0 for _ in 0..n] for _ in 0..n]  // expect: nested_list_grid
for i in 0..n:
    for j in 0..n:
        g[i][j] = i * j % 7  // expect: unproven_list_index
var s = 0
for i in 0..n:
    for j in 0..n:
        s += g[i][j]
print(s)
