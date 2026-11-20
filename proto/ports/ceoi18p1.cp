// cp: 0.1 af7fc88+dirty
// CEOI 2018 Cloud Computing: buy machines (cores, clock rate, price), then sell
// orders (cores, minimum rate, price). Sweep rates from fast to slow so every
// machine an order may use is already in the pool; knapsack on spare cores.
type Offer = (cores: 1..=50, rate: 1..=1e9, price: 1..=1e9)
let n: 1..=2000 = read()
let machines: [n]Offer = read()
let m: 1..=2000 = read()
let orders: [m]Offer = read()

type Deal = (rate: Int, cores: Int, gain: Int)
var deals: List<Deal> = []
for o in machines: deals.push((o.rate, o.cores, -o.price))
for o in orders: deals.push((o.rate, -o.cores, o.price))
// fastest first; at equal rate machines (positive cores) come before orders
sort(deals, by: d => (-d.rate, -d.cores))

domain Spare = 0..=n * 50
var best: [Spare]?Int = none        // best profit leaving this many spare cores
best[Spare(0)] = 0
for d in deals:
    let before = best               // each deal is taken at most once
    for c in Spare:
        if let b = before[c] and let left = c + d.cores: best[left] max= b + d.gain
print(max(best)!)
