// expect: ok, output "0 2 1"
// `track`'s element type comes from the sequence it walks, not from its state: `x` is a `(w: Int)` here, so `x.w` checks (rebuild round 33 F1).
let a = [(w: 1), (w: 2), (w: 3)]
let best = a.minimal(state: track(0, (mut s, x) => s += x.w, (mut s, x) => s -= x.w), ok: t => t.state >= 3)
print([if let p = q: p.len else: 0 for q in best])
