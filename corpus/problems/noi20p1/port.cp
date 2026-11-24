// Source: NOI 2020 P1 (Delicacy), https://dmoj.ca/problem/noi20p1
// limits: 2000 ms, 512 MB
// Problem: n <= 50 cities with joy c_i, m <= 501 one-way roads taking w <= 5 days. A walk
//   leaves city 1 on day 0 and must reach city 1 on exactly day T <= 1e9, never waiting; each
//   arrival (start and end too) earns the city's joy, and k <= 200 festivals (distinct days
//   t_i) add y_i <= 1e9 for being in city x_i on day t_i. Print the most joy, or -1.
// Solution: split each city into 5 states (city, days still to travel), so one day is one
//   max-plus matrix over 5n states whose missing entries are `None`. Precompute its 2^b powers
//   (O((5n)^3 log T)); between consecutive festivals advance the best-joy row by the bits of
//   the gap (O((5n)^2 log T) each) and add the bonus at the festival's city. The answer is the
//   row at city 1 after day T.
// original: corpus/submissions/3890258.cpp
// Bounds: n: 1..=50; m: 1..=501; t: 1..=1e9; k: 0..=200; joy: 1..=52501; roads.w: 1..=5;
//   fests.day: 1..=t; fests.bonus: 1..=1e9; roads.u, roads.v, fests.city: 1..=n

let (n, m, t, k): (1..=50, 1..=501, 1..=1e9, 0..=200) = read()
domain City = 1..=n
domain State = 0..5 * n  // (city, days still to travel)
let joy: [City]1..=52501 = read()
let roads: [m](u: City, v: City, w: 1..=5) = read()
let fests: [k](day: 1..=t, city: City, bonus: 1..=1e9) = read()
type Row = [State]?Int
type Mat = [State]Row

fn at(c: City, wait: Int) -> State = State.nth(c.ord * 5 + wait)

fn mul(a: Mat, b: Mat) -> Mat:
    [State] i =>
        var c: Row = None
        for j in State:
            if let via = a[i][j]:
                let bj = b[j]
                for l in State:
                    if let w = bj[l]: c[l] max= Some(via + w)
        c

fn apply(x: Row, a: Mat) -> Row:
    var y: Row = None
    for i in State:
        if let have = x[i]:
            let ai = a[i]
            for j in State:
                if let w = ai[j]: y[j] max= Some(have + w)
    y

var day: Mat = [State] _ => [State] _ => None
for r in roads: day[at(r.u, 0)][at(r.v, r.w - 1)] = Some(joy[r.v])
for c in City:
    for wait in 1..5: day[at(c, wait)][at(c, wait - 1)] = Some(0)

// pows[b] is 2^b days
var pows = [day]
while 2 ** pows.len <= t: pows.push(mul(pows.last!, pows.last!))

let home = at(City.first!, 0)
var best: Row = None
best[home] = Some(joy[City.first!])
var now = 0

fn advance(to: Int):
    for b in 0..pows.len:
        if (to - now) / 2 ** b % 2 == 1: best = apply(best, pows[b])
    now = to

for f in fests.collect().sorted():
    advance(f.day)
    let s = at(f.city, 0)
    if let v = best[s]: best[s] = Some(v + f.bonus)
advance(t)
print(best[home] ?? -1)
