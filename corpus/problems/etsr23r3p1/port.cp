// Source: ETSR '23 R3 P1, https://dmoj.ca/problem/etsr23r3p1
// Problem: n operations act on a number v that starts at a: add x, subtract x, multiply by x,
//   or add x * a; after each one v is clamped to lo..=hi. Print the final v for each of q
//   starting values a.
// Solution: every operation and the clamp are non-decreasing in a, so over the distinct starts
//   in increasing order the values stay sorted. A range-tag tree over the starts holds each one's
//   composed map v -> mul * v + m * a + b (a clamp to c is mul = m = 0, b = c). After each
//   operation the clamp sets a prefix to lo and a suffix to hi, both found by binary search on
//   point reads: O(n log^2 q). The maps' coefficients wrap mod 2^64 as U64: a start that is
//   never clamped can multiply past Int while its value stays in lo..=hi, and the value comes
//   out right mod 2^64.
// original: corpus/submissions/5527625.cpp
// The statement is not public: the bounds are the reference's ints, and it is right only on
// non-negative values (a negative start can leave lo..=hi in it).
type Value = 0..=2147483647

struct Step(mul: U64, m: U64, b: U64):
    fn then(self, next: Step) -> Step =
        Step(next.mul * mul, next.mul * m + next.m, next.mul * b + next.b)

fn set(c: Int) -> Step = Step(U64(0), U64(0), U64(c))

let (n, lo): (1..=1e6, Value) = read()
let hi: lo..=2147483647 = read()
let ops: [0..n](kind: 1..=4, x: Value) = read()
let q: 1..=1e6 = read()
let starts: [0..q]Value = read()

let (domain Start, start) = compress(starts)
type Held = (a: U64, v: U64)  // a start and its value, mod 2^64
let onto = Action<Held, Step>(
    id: Step(U64(1), U64(0), U64(0)),
    compose: (newer, older) => older.then(newer),
    apply: (f, h, _) => (a: h.a, v: f.mul * h.v + f.m * h.a + f.b),
    over: Monoid((a: U64(0), v: U64(0)), (x, _) => x),
)
var held = SegTree(act: onto, a: [Start] s => (a: U64(start[s]), v: U64(start[s])))

// Before its clamp a value lies in -2^31..2^62, so the 2^31 shift reads it back from U64.
fn value(s: Start) -> Int = Int(held.get(s).v + U64(1 << 31)) - (1 << 31)

for op in ops:
    let x = U64(op.x)
    held.apply_all(match op.kind:
        1 => Step(U64(1), U64(0), x)
        2 => Step(U64(1), U64(0), U64(0) - x)
        3 => Step(x, U64(0), U64(0))
        _ => Step(U64(1), x, U64(0)))
    held.apply(span(Start.all.l, Start.all.bsearch(s => value(s) >= lo)), set(lo))
    held.apply(span(Start.all.bsearch(s => value(s) > hi), Start.all.r), set(hi))

for a in starts: print(value(start.index_of(a)))
