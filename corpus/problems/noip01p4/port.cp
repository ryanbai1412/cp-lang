// NOIP 2001 Car's travel route: every city is a rectangle with an airport on
// each corner, three of them given. Rail joins the airports of one city at
// the city's price per unit; a flight joins any two airports of different
// cities at price `air` per unit. Print the cheapest trip from city `from`
// to city `to`, to two decimals.
// Solution: complete the fourth corner from the right angle, then run
// Floyd-Warshall over every (city, corner) airport.
// original: corpus/submissions/5086731.cpp
use std.geometry.{dot}  // WISH F1
use std.graph.{floyd_warshall}

let n: 1..=100 = read()
let air: 1..=1000 = read()
domain City = 1..=n
let from: City = read()
let to: City = read()
let given: [City](a: Pt, b: Pt, c: Pt, rail: 1..=1000) = read()

// The corner opposite a right angle at p is q + r - p.
fn fourth(a: Pt, b: Pt, c: Pt) -> Pt:
    for (p, q, r) in [(a, b, c), (b, c, a), (c, a, b)]:
        if dot(q - p, r - p) == 0: return q + r - p
    unreachable  // WISH G17

domain Corner = 1..=4
type Airport = (City, Corner)
let corners = [City] c => do:
    let s = given[c]
    [s.a, s.b, s.c, fourth(s.a, s.b, s.c)]
fn at(u: Airport) -> Pt:
    let (c, k) = u
    corners[c][k.ord]

fn price(u: Airport, v: Airport) -> F64:  // WISH F15
    let ((cu, _), (cv, _)) = (u, v)
    F64(if cu == cv: given[cu].rail else: air) * at(u).dist(at(v))
var cost: [Airport, Airport]?F64 = [Airport, Airport] (u, v) => Some(price(u, v))
assert floyd_warshall(mut cost)
print([cost[(from, i), (to, j)]! for i in Corner for j in Corner].min()!, precision: 2)
