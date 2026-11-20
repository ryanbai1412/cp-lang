// CEOI 2018 Cloud Computing: buy machines (cores, clock rate, price), then sell
// orders (cores, minimum rate, price). Sweep rates from fast to slow so every
// machine an order may use is already in the pool; knapsack on spare cores.
type Offer = (cores: 1..=50, rate: 1..=1e9, price: 1..=1e9)
let n: 1..=2000 = read()
let machines: [0..n]Offer = read()
let m: 1..=2000 = read()
let orders: [0..m]Offer = read()

type Deal = (rate: Int, cores: Int, gain: Int)
var deals: List<Deal> = [(o.rate, o.cores, -o.price) for o in machines]
for o in orders: deals.push((o.rate, -o.cores, o.price))
// fastest first; at equal rate machines (positive cores) come before orders
sort(mut deals, by: d => (-d.rate, -d.cores))

let top = 50 * n
var best: [0..=top]?Int = none          // best profit leaving this many spare cores
best[0] = 0
for (_, cores, gain) in deals:
    // each deal is taken at most once: walk away from where the cores come from
    if cores > 0:
        for c in (cores..=top).rev: best[c] max= best[c - cores] + gain
    else:
        for c in 0..=top + cores: best[c] max= best[c - cores] + gain
var profit = 0
for b in best: profit max= b
print(profit)
