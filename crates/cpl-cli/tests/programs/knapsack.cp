// 0/1 knapsack over capacities, then the items of one best choice.
let n: 1..=100 = read()
let cap: 1..=10000 = read()
domain I = 1..=n
let items: [I](w: 1..=10000, v: 0..=1000000000) = read()
var best: [I.bounds, 0..=cap]Int = 0
for i in I:
    for c in 0..=cap:
        best[i.after, c] = best[i.before, c]
        if c >= items[i].w:
            best[i.after, c] max= best[i.before, c - items[i].w] + items[i].v
print(best[I.end, cap])
var chosen: List<I> = []
var left = cap
for i in I.rev:
    if best[i.after, left] != best[i.before, left]:
        chosen.push(i)
        left -= items[i].w
chosen.reverse()
print(chosen)
print(sum(items[i].v for i in chosen), sum(items[i].w for i in chosen))
