let (n, cap): (1..=100, 1..=1e5) = read()
let items: [0..n](w: 1..=1e5, v: 1..=1e9) = read()
var best: [0..=cap]Int = 0
for it in items:
    for c in (it.w..=cap).rev:
        best[c] max= best[c - it.w] + it.v
print(best[cap])
