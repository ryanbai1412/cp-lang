struct Item(id: Int, prio: Int)

impl Ord for Item:
    fn lt(a, b) = if a.prio != b.prio: a.prio > b.prio else: a.id < b.id

let xs = [Item(1, 10), Item(2, 30), Item(3, 20), Item(4, 30)]
for x in xs.sorted():
    print(x.id, x.prio)
for x in xs[1..4].sorted():
    print(x.id, x.prio)
let lo = min(xs[0], xs[1], xs[2])
let hi = max(xs[0], xs[1], xs[2])
print(lo.id, hi.id)
