// Source: NOI 2020 P1 (Delicacy), https://dmoj.ca/problem/noi20p1
// Problem: n <= 50 cities with joy c_i, m <= 501 one-way roads taking w <= 5 days. A walk
//   leaves city 1 on day 0 and must reach city 1 on exactly day T <= 1e9, never waiting; each
//   arrival (start and end too) earns the city's joy, and k <= 200 festivals (distinct days
//   t_i) add y_i <= 1e9 for being in city x_i on day t_i. Print the most joy, or -1.
// Solution: split each city into 5 states (city, days still to travel), so one day is one
//   max-plus matrix over 5n states. Precompute its 2^b powers (O((5n)^3 log T)); between
//   consecutive festivals advance the best-joy row vector by the bits of the gap
//   (O((5n)^2 log T) each) and add the bonus at the festival's city. The walk's answer is
//   the row at city 1 after day T.
// original: corpus/submissions/3890258.cpp
const NONE = -4e18  // "no walk": the zero of max-plus

let (n, m, t, k): (Int, Int, Int, Int) = read()
domain City = 1..=n
domain State = 0..5 * n  // (city, days still to wait)
domain Road = 0..m
domain Fest = 0..k
let joy: [City]Int = read()
let roads: [Road](u: City, v: City, w: Int) = read()
let fests: [Fest](day: Int, city: City, bonus: Int) = read()
type Mat = [State, State]Int
type Row = [State]Int

fn at(c: City, wait: Int) -> State = State(c.to_idx * 5 + wait)

fn mul(a: Mat, b: Mat) -> Mat:
    var c: Mat = [State, State] (i, j) => NONE
    for i in State:
        for j in State:
            if a[i, j] > NONE:
                for l in State:
                    if b[j, l] > NONE: c[i, l] = max(c[i, l], a[i, j] + b[j, l])
    c

fn apply(x: Row, a: Mat) -> Row:
    var y: Row = [State] i => NONE
    for i in State:
        if x[i] > NONE:
            for j in State:
                if a[i, j] > NONE: y[j] = max(y[j], x[i] + a[i, j])
    y

var day: Mat = [State, State] (i, j) => NONE
for r in roads: day[at(r.u, 0), at(r.v, r.w - 1)] = joy[r.v]
for c in City:
    for wait in 1..5: day[at(c, wait), at(c, wait - 1)] = 0

// pows[b] is 2^b days
var pows = [day]
while 2 ** pows.len <= t: pows.push(mul(pows.last!, pows.last!))

var best: Row = [State] i => NONE
best[at(City(1), 0)] = joy[City(1)]
var now = 0
for f in ([f for f in fests] + [(day: t, city: City(1), bonus: 0)]).sorted():
    for b in 0..pows.len:
        if (f.day - now) / 2 ** b % 2 == 1: best = apply(best, pows[b])
    now = f.day
    if best[at(f.city, 0)] > NONE: best[at(f.city, 0)] += f.bonus
print(if best[at(City(1), 0)] > NONE: best[at(City(1), 0)] else: -1)
