// CEOI 2018 "Cloud Computing": buy computers (cores, clock, price) and accept
// orders (cores, min clock, pay). Going down by clock, a core bought at a clock
// can serve any order at or below it: a knapsack over free cores.
let n = read<1..=2000>()
let comps = read<[n](cores: 1..=50, clock: 1..=1e9, price: 1..=1e9)>()
let m = read<1..=2000>()
let orders = read<[m](cores: 1..=50, clock: 1..=1e9, pay: 1..=1e9)>()
let deals = chain(
    comps.map(c => (clock: c.clock, order: false, cores: c.cores, cash: -c.price)),
    orders.map(o => (clock: o.clock, order: true, cores: -o.cores, cash: o.pay)),
).sorted(by: d => (desc(d.clock), d.order))

domain Free = 0..=comps.sum(of: .cores)
var best: [Free]?Int = [Free] f => if f == 0: 0 else: None
for d in deals:
    let before = best.clone()
    for f in Free:
        if let g = Free.at(Int(f) - d.cores), let p = before[g]: best[f] max= Some(p + d.cash)
print(Free.filter_map(f => best[f]).max()!)
