// Source: NOIP 2001 Senior P4 (Car's Travel Route), https://dmoj.ca/problem/noip01p4
// Problem: n <= 100 cities, each a rectangle with an airport on every corner, three of them
//   given. Rail between airports of one city costs the city's price per unit length; a flight
//   between airports of different cities costs `air` per unit. Print the cheapest trip from
//   city `from` to city `to`, any airports at either end, to two decimals.
// Solution: the fourth corner is q + r - p, where p is the given corner with the right angle.
//   Then Floyd-Warshall over all 4n airports with Euclidean prices, O(n^3), and take the
//   cheapest of the 16 corner pairs.
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
