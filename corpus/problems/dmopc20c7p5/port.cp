// Source: DMOPC '20 Contest 7 P5 (Mayors and Tolls), https://dmoj.ca/problem/dmopc20c7p5
// limits: 2000 ms, 256 MB
// Problem: N <= 500 cities with mayor fees b_i and M <= 500 roads (u, v, p): a road earns p if
//   both its cities' mayors are paid (values <= 1e9). Print the best total profit minus fees.
// Solution: project selection: each city has profit -b_i, each road profit p and requires both
//   its cities, so the answer is the maximum-weight closure of that dependency graph
//   (`std.flow.max_closure`, a min cut) over N + M nodes.
// Bounds: n: 1..=500; m: 0..=500; fee, roads.toll: 0..=1e9; roads.u, roads.v: 1..=n

// original: corpus/submissions/4370353.cpp
let n: 1..=500 = read()
let m: 0..=500 = read()
domain City = 1..=n
domain Road = 0..m
let fee: [City]0..=1e9 = read()
let roads: [Road](u: City, v: City, toll: 0..=1e9) = read()

// One index space for both kinds of choice until sum domains (`City + Road`) land.
domain Choice = 0..n + m
fn city(c: City) -> Choice = Choice(c.ord)
fn road(r: Road) -> Choice = Choice(n + r.ord)

var profit: [Choice]Int = 0
for c in City: profit[city(c)] = -fee[c]
for r in Road: profit[road(r)] = roads[r].toll
// a road needs both of its cities paid
var needed_by: [Choice]List<Choice> = []
for r in Road:
    needed_by[city(roads[r].u)].push(road(r))
    needed_by[city(roads[r].v)].push(road(r))
print(max_closure(needed_by, profit: x => profit[x]).value)
