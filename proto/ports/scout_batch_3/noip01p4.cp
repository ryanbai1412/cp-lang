// NOIP 2001 Car's travel route: every city is a rectangle with an airport on
// each corner, three of them given. Rail joins the airports of one city at
// the city's price per unit; a flight joins any two airports of different
// cities at price `air` per unit. Print the cheapest trip from city `from`
// to city `to`, to one decimal.
use std.geometry.{Pt, dot, dist}                                    // WISH F1; dist is G15
use std.graph.{all_pairs}                                           // WISH G16

@multitest
fn solve():
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
        unreachable                                                 // WISH G17

    domain Corner = 1..=4
    type Airport = (city: City, corner: Corner)                     // WISH G18: a record of domains as an index
    let at: [Airport]Pt = [City] c => do:
        let s = given[c]
        [s.a, s.b, s.c, fourth(s.a, s.b, s.c)]

    fn price(u: Airport, v: Airport) -> F64:                        // WISH F15
        (if u.city == v.city: given[u.city].rail else: air) * dist(at[u], at[v])
    let cost = all_pairs([Airport, Airport] (u, v) => price(u, v))  // WISH G16
    print(min(cost[(from, i), (to, j)] for i in Corner for j in Corner), digits: 1)   // WISH G4
