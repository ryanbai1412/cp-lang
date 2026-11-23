// CF 1990D: shortest path over (row, carried) where carried says which half of the row
// a 2x2 from the row above already whitened: 0 none, 1 columns 1-2, 2 columns 3-4.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]0..=n>()
domain Row = 0..=n
domain Carried = 0..3

fn moves(i: Row, s: Carried) -> yields (to: (Row, Carried), w: Int):
    if Int(i) == n: return
    let nx = i.next!
    let black = a[Int(i)]
    yield (to: (nx, Carried(0)), w: 1)
    if black == 0:
        yield (to: (nx, Carried(0)), w: 0)
    elif black <= 2:
        if s == Carried(1): yield (to: (nx, Carried(0)), w: 0)
        else: yield (to: (nx, Carried(1)), w: 1)
    elif black <= 4:
        if s == Carried(1): yield (to: (nx, Carried(2)), w: 1)
        elif s == Carried(2): yield (to: (nx, Carried(1)), w: 1)

let p = dijkstra((v: (Row, Carried)) => moves(v.0, v.1), from: [((Row.first!, Carried(0)), 0)])
print(p[(Row.last!, Carried(0))]!)
