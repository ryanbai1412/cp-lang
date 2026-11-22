let (n, x) = read<(1..=20, 1..=1e9)>()
let w = read<[n]1..=x>()
domain Mask = 0..(1 << n)
var best = [Mask] s => (rides: n + 1, load: 0)
best[Mask(0)] = (rides: 1, load: 0)
for s in Mask:
    let cur = best[s]
    for i in 0..n:
        if (Int(s) >> i) & 1 == 0:
            let t = Mask(Int(s) | (1 << i))
            if cur.load + w[i] <= x:
                best[t] min= (rides: cur.rides, load: cur.load + w[i])
            else:
                best[t] min= (rides: cur.rides + 1, load: w[i])
print(best[Mask((1 << n) - 1)].rides)
