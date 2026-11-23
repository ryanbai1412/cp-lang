// cf1249e By Elevator or Stairs?: shortest path over (floor, mode).
// Mode 0 is on the stairs/landing, mode 1 is inside the elevator; boarding costs c.
let n: 2..=2e5 = read()
let c: 1..=1000 = read()
domain V = 1..=n
let a = read<[n - 1]1..=1000>()
let b = read<[n - 1]1..=1000>()
domain Mode = 0..2

fn moves(v: V, k: Mode) -> yields (to: (V, Mode), w: Int):
    if k == Mode(0): yield (to: (v, Mode(1)), w: c)
    else: yield (to: (v, Mode(0)), w: 0)
    if let u = v.next:
        let i = Int(v) - 1
        yield (to: (u, k), w: if k == Mode(0): a[i] else: b[i])

let p = dijkstra(s => moves(s.0, s.1), from: [((V.first!, Mode(0)), 0)])
print([V] v => p.dist[(v, Mode(0))]!)
