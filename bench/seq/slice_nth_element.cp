let n = 200_000
let base: List<Int> = [((i * 1103515245 + 12345) % 2147483647) for i in 0..n]
var xs: List<Int> = [0 for _ in 0..n]
var acc = 0
for r in 0..80:
    let mask = (r * 2654435761) % 2147483647
    for i in 0..n: xs[i] = base[i] ^ mask
    let l = (r * 3713) % (n / 2)
    let hi = l + n / 2
    let k = (r * 1009) % (n / 2)
    select(mut xs[l..hi], k)
    acc += xs[l + k]

print(acc)
