let n: 1..=50 = read()
let cap: 1..=1000 = read()
domain I = 1..=n
let items: [I](w: 1..=1000, v: 0..=1e9) = read()

var best: [I.bounds, 0..=cap]Int = 0
for i in I:
    for c in 0..=cap:
        best[i.r, c] = best[i.l, c]
        if c >= items[i].w:
            best[i.r, c] max= best[i.l, c - items[i].w] + items[i].v
print(best[I.last.r, cap])
