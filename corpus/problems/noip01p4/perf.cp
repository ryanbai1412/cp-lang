// NOIP 2001 P4, tuned: port.cp with the all-pairs closure written out over plain F64
// weights (every pair has a hop), as orig.cpp's Floyd-Warshall, instead of `closure()` on
// an ?F64 matrix and its copies.
//
// Source: NOIP 2001 Senior P4 (Car's Travel Route), https://dmoj.ca/problem/noip01p4
// Problem: n <= 100 cities, each a rectangle with an airport on every corner, three of them
//   given. Rail between airports of one city costs the city's price per unit length; a flight
//   between airports of different cities costs `air` per unit. Print the cheapest trip from
//   city `from` to city `to`, any airports at either end, to two decimals.
// Solution: the fourth corner is q + r - p, where p is the given corner with the right angle.
//   Every airport pair is one hop at its price; the all-pairs closure (Floyd-Warshall over the
//   4n airports, O(n^3)) gives the trip, the cheapest of the 16 corner pairs.
// original: corpus/submissions/5086731.cpp
// Bounds: n: 1..=100; air, city.rail: 1..=1000; from, to: 1..=n
let n: 1..=100 = read()
let air: 1..=1000 = read()
domain City = 1..=n
let (from, to): (City, City) = read()
let city: [City](a: Pt, b: Pt, c: Pt, rail: 1..=1000) = read()

domain Corner = 0..4
type Airport = (City, Corner)
// the corner opposite the right angle at p is q + r - p
let corner = [Airport] ((c, k)) => do:
    let (a, b, c3, _) = city[c]
    let (p, q, r) = [(a, b, c3), (b, c3, a), (c3, a, b)].find(((p, q, r)) => dot(q - p, r - p) == 0)!
    [a, b, c3, q + r - p][k.ord]

fn price(u: Airport, v: Airport) -> F64 =
    F64(if u.0 == v.0: city[u.0].rail else: air) * corner[u].dist(corner[v])
var trip = [Airport, Airport] (u, v) => price(u, v)
for k in Airport:
    for i in Airport:
        let via = trip[i, k]
        for j in Airport: trip[i, j] min= via + trip[k, j]
print(min(trip[(from, i), (to, j)] for i in Corner for j in Corner)!, precision: 2)
