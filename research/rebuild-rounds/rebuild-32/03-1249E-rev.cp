// cf1249e revised: the elevator overhead c is paid only on the first boarding.
// Mode 0: on foot, never boarded; 1: inside the elevator; 2: on foot, already paid.
let n: 2..=2e5 = read()
let c: 1..=1000 = read()
domain V = 1..=n
let a = read<[n - 1]1..=1000>()
let b = read<[n - 1]1..=1000>()
domain Mode = 0..3

fn moves(v: V, k: Mode) -> yields (to: (V, Mode), w: Int):
    if k == Mode(0): yield (to: (v, Mode(1)), w: c)
    elif k == Mode(2): yield (to: (v, Mode(1)), w: 0)
    else: yield (to: (v, Mode(2)), w: 0)
    if let u = v.next:
        let i = Int(v) - 1
        yield (to: (u, k), w: if k == Mode(1): b[i] else: a[i])

let p = dijkstra(s => moves(s.0, s.1), from: [((V.first!, Mode(0)), 0)])
print([V] v => min(p.dist[(v, Mode(0))]!, p.dist[(v, Mode(2))]!))
