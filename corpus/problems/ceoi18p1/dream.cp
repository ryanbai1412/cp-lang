// CEOI 2018 "Cloud Computing": buy computers (cores, clock, price) and accept
// orders (cores, min clock, pay). Going down by clock, a core bought at a clock
// can serve any order at or below it: a knapsack over free cores.
let comps = read<List<(cores: 1..=50, clock: 1..=1e9, price: 1..=1e9)>>()
let orders = read<List<(cores: 1..=50, clock: 1..=1e9, pay: 1..=1e9)>>()
let deals = chain(
    comps.map(c => (clock: c.clock, order: false, cores: c.cores, cash: -c.price)),
    orders.map(o => (clock: o.clock, order: true, cores: -o.cores, cash: o.pay)),
).sorted(by: d => (desc(d.clock), d.order))

domain Free = 0..=comps.sum(of: .cores)
var best: [Free]?Int = [Free] f => (f == 0).then(0)
for d in deals:
    best = [Free] f => max(best[f], Free.at(f - d.cores).and_then(g => best[g]).map(p => p + d.cash))
print(best.values.flatten().max()!)
