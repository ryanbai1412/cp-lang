var f: [0..=60, 0..=60, 0..=60]Int = 0
for k in 1..=60:
    for a in 0..=k:
        for b in 0..=a:
            f[k, a, b] = f[k - 1, a, b] + 1
print(f[60, 3, 1])
fn corners(n: Int) -> Int:
    var c: [0..n, 0..n, 0..n, 0..n]Int = 0
    for i in 0..n:
        for j in 0..n:
            c[i, j, i, j] += i * n + j
            c[j, i, j, i] -= 1
    return c[n - 1, 1, n - 1, 1] + c[1, 0, 1, 0]
print(corners(4))
var names: [0..2, 0..2, 0..2]Str = ""
names[1, 0, 1] = "mid"
print(names[1, 0, 1], names[0, 0, 0].len)
var opt: [0..2, 0..2, 0..2]?Int = None
opt[1, 1, 1] = 5
print(opt[1, 1, 1] ?? 0, opt[0, 1, 1] ?? -1)
