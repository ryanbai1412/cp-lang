let n = 1_000_000
var acc = 0
for r in 0..4:
    let xs = [(cost: (i * 7_919_993 + r) % 1_000_000_007, v: i, p: i / 2) for i in 0..n].sorted(by: j => desc(j.cost))
    acc += xs[0].v + xs[n - 1].v + xs[n / 2].p
print(acc)
