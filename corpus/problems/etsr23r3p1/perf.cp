// ETSR '23 R3 P1, tuned: port.cp's range-tag tree folds to its last start instead of its first,
// so the sorted values let `right_while` find each clamp's cut in one O(log q) descent instead of
// a binary search over point reads. O(n log q).
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
type Held = (a: U64, v: U64, any: Bool)  // a start and its value, mod 2^64; `any` is false on the identity
let last = Monoid((a: U64(0), v: U64(0), any: false), (x: Held, y: Held) => if y.any: y else: x)
let onto = Action<Held, Step>(
    id: Step(U64(1), U64(0), U64(0)),
    compose: (newer, older) => older.then(newer),
    apply: (f, h, _) => (a: h.a, v: f.mul * h.v + f.m * h.a + f.b, any: h.any),
    over: last,
)
var held = SegTree(last, act: onto, a: [Start] s => (a: U64(start[s]), v: U64(start[s]), any: true))

// Before its clamp a value lies in -2^31..2^62, so the 2^31 shift reads it back from U64.
fn value(h: Held) -> Int = Int(h.v + U64(1 << 31)) - (1 << 31)

for op in ops:
    let x = U64(op.x)
    held.apply_all(match op.kind:
        1 => Step(U64(1), U64(0), x)
        2 => Step(U64(1), U64(0), U64(0) - x)
        3 => Step(x, U64(0), U64(0))
        _ => Step(U64(1), x, U64(0)))
    held.apply(span(Start.all.l, held.right_while(Start.all.l, h => not h.any or value(h) < lo)), set(lo))
    held.apply(span(held.right_while(Start.all.l, h => not h.any or value(h) <= hi), Start.all.r), set(hi))

for a in starts: print(value(held.get(start.index_of(a))))
