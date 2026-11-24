// Median constraints: find numbers in 1..=1e9 with median(a[i], a[j], a[k]) = x for every
// fact, or say NO. For each slot and value a fact names, keep two facts: "a[s] < x" and
// "a[s] > x". At most one of a fact's three numbers is below x and at most one above, a
// number is never both, and along one slot's values in increasing order a number is
// above this one or below the next, exactly one. That is 2-SAT.
enum Side: Below | Above
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain Slot = 1..=n
let facts: [m](i: Slot, j: Slot, k: Slot, x: 1..=1e9) = read()
let (domain Pin, pins) = compress([(s:, x: f.x) for f in facts for s in [f.i, f.j, f.k]])
domain Claim = (Pin, Side)
fn claim(p: Pin, side: Side) -> Lit<Claim> = Lit(Claim((p, side)), true)

var sat = TwoSat<Claim>()
for f in facts:
    let ps = [pins.index_of((s:, x: f.x)) for s in [f.i, f.j, f.k]]
    for side in Side: sat.at_most_one([claim(p, side) for p in ps])
for p in Pin: sat.add_or(claim(p, Below).neg, claim(p, Above).neg)
for (p, q) in Pin.collect().adjacent if pins.get(p).s == pins.get(q).s:
    sat.add_xor(claim(p, Above), claim(q, Below))

match sat.solve():
    None => print("NO")
    Some(truth) =>
        print("YES")
        var number: [Slot]Int = 1
        for p in Pin:
            let pin = pins.get(p)
            if truth[Claim((p, Above))]: number[pin.s] max= pin.x + 1
            elif not truth[Claim((p, Below))]: number[pin.s] max= pin.x
        print(number)
