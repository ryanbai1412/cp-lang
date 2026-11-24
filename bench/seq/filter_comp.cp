let n = 20_000_000
let ks = [(cost: i * 3, v: i, p: i / 2) for i in 0..n if (i * 7_919_993) % 16 < 5]
print(ks.len + ks[0].cost + ks[ks.len - 1].p)
