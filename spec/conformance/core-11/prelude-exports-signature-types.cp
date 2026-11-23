// expect: ok, output "3 3 2 2"
// rule: 11.3.2
// A type a prelude signature names is in the prelude: `a.ring.minimal` yields `RingSlice`s, so a lambda can annotate one with no `use` (rebuild round 34 F1).
domain M = 0..4
let a: [M]Int = [3, 1, 4, 1]
let f = (w: RingSlice<M, Int>) => w.len
print([if let w = p: f(w) else: 0 for p in a.ring.minimal(Window(+), ok: t => (t.all ?? 0) >= 5)])
