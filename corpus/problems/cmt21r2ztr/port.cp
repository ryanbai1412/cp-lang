// Source: CMT 2021 R2 (ztr), https://dmoj.ca/problem/cmt21r2ztr
// Problem: n <= 1e5 hidden numbers and m <= 1e5 facts, each saying that the median of
//   numbers i, j, k is x (x <= 1e9). Print YES and numbers that fit every fact, or NO.
// Solution: a fact says at most one of its three numbers is below x and at most one is
//   above, so the only questions are "is number s below / above value x" for each slot
//   and value a fact names. These are 2-SAT variables: each fact allows at most one of
//   its three below (or above), a mark is never both, and consecutive marks of one slot
//   chain (above this or below the next, exactly one). `TwoSat` decides it and the
//   truth gives each slot its bound, O(n + m log m).
// original: corpus/submissions/4016530.cpp
// checker: proto/ports/scout_batch_9/harness/cmt21r2ztr_check.cpp
// Bounds: n, m: 1..=1e5; facts.i, facts.j, facts.k: 1..=n; facts.x: 1..=1e9
enum Side: Below | Above
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain Slot = 1..=n
let facts: [m](i: Slot, j: Slot, k: Slot, x: 1..=1e9) = read()

// Every slot and value some fact puts together, in order of slot, then value.
let (domain Pin, pins) = compress([(s:, x: f.x) for f in facts for s in [f.i, f.j, f.k]])
// One question for each pin and side: is the number below (above) the pin's value?
domain Question = (Pin, Side)
fn on(p: Pin, side: Side) -> Lit<Question> = Lit(Question((p, side)), true)

var sat = TwoSat<Question>()
for f in facts:
    let ps = [pins.index_of((s:, x: f.x)) for s in [f.i, f.j, f.k]]
    for side in Side: sat.at_most_one([on(p, side) for p in ps])
for p in Pin: sat.add_or(on(p, Below).neg, on(p, Above).neg)
// A number strictly between two values of its slot can move down onto one, so it is
// above the lower value or below the higher one, and never both.
for (p, q) in Pin.collect().adjacent if pins.get(p).s == pins.get(q).s:
    sat.add_xor(on(p, Above), on(q, Below))

if let truth = sat.solve():
    print("YES")
    var number: [Slot]Int = 1
    for p in Pin:
        let pin = pins.get(p)
        if truth[Question((p, Above))]: number[pin.s] max= pin.x + 1
        elif not truth[Question((p, Below))]: number[pin.s] max= pin.x
    print(number)
else:
    print("NO")
