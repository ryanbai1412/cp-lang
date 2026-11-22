// 0/1 knapsack: the best total value of items that fit in `cap`, each
// item taken at most once. A bug is planted in the capacity loop.
let (n, cap): (1..=6, 0..=12) = read()
domain Item = 1..=n
let items: [Item](w: 1..=6, v: 1..=9) = read()

var best: [0..=cap]Int = 0
for i in Item:
    let (w, v) = (items[i].w, items[i].v)
    for c in 0..=cap:
        if c >= w: best[c] max= best[c - w] + v
print(best[cap])

@brute(main)
fn every_subset():
    var answer = 0
    for mask in 0..(1 << n):
        var (w, v) = (0, 0)
        for i in Item:
            if (mask >> i.to_idx) & 1 == 1:
                w += items[i].w
                v += items[i].v
        if w <= cap: answer max= v
    print(answer)
