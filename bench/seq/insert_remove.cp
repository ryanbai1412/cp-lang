let n = 100_000
var v: List<Int> = []
var seed = 987654321
var acc = 0
for i in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483647
    v.insert(0, seed % 1000003)
    if i % 4 == 3:
        acc += v.remove(v.len / 2)
acc += v[0] + v[v.len - 1] + v.len
print(acc)
