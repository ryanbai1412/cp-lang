let n = 200_000
var v: List<Int> = [0 for _ in 0..n]
var seed = 987654321
for i in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483647
    v[i] = seed % 1000003

var acc = 0
for r in 0..200:
    let k = (r * 131071 + 37) % n
    v.rotate_left(k)
    acc += v[0] + v[n / 2] + v[n - 1]
    v.rotate_right(k / 3)
    acc += v[1] + v[n - 2]

print(acc)
