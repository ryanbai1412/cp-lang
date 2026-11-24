// DMOPC '20 C7 P5 (dream): cities with fees, roads (u, v, toll) earning toll when both cities
// are paid. Maximize tolls minus fees. Project selection: a road is profit toll, a city is
// profit -fee, and taking a road needs both its cities, so the answer is the maximum closure of
// the needs graph. Cities and roads share one index space, roads after the cities.
let (n, m) = read<(1..=500, 0..=500)>()
domain City = 1..=n
domain Road = 0..m
let fee: [City]0..=1e9 = read()
let roads: [Road](u: City, v: City, toll: 0..=1e9) = read()

domain Choice = 0..n + m
fn city(c: City) -> Choice = Choice(c.ord)
fn road(r: Road) -> Choice = Choice(n + r.ord)

var profit: [Choice]Int = 0
var needed_by: [Choice]List<Choice> = []
for c in City: profit[city(c)] = -fee[c]
for (r, (u, v, toll)) in roads.items:
    profit[road(r)] = toll
    for c in [u, v]: needed_by[city(c)].push(road(r))
print(max_closure(needed_by, profit: x => profit[x]).value)
