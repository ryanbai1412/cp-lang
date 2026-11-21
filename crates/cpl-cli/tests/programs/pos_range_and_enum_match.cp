// Position `back(k)` / `ahead(k)`, `Range<D>.drop(k)` / `.take(k)`,
// unit-variant `is` test, and `match` on an enum with a payload variant.
domain D = 1..=5
let p = D(3)
print(p.back(0)!, p.back(2)!, p.back(3) == none)
print(p.ahead(0)!, p.ahead(2)!, p.ahead(3) == none)

let r: Range<D> = D(1)..=D(5)
print([x for x in r.drop(2)])
print([x for x in r.take(3)])
print([x for x in r.drop(1).take(2)])
print([x for x in r.drop(10)], [x for x in r.take(0)], [x for x in r.take(-1)])
print([i for i in (10..15).drop(2).take(2)])

struct Walk:
    z: Int
    o: Int

enum Phase:
    Seeking
    Split(base: Walk)

fn describe(ph: Phase) -> Int:
    let seeking = ph is Seeking
    let v = match ph:
        Seeking => 0
        Split(base) => base.z * 10 + base.o
    if seeking: v - 1 else: v

print(describe(Phase.Seeking), describe(Phase.Split(Walk(z: 3, o: 7))))
let p1: Phase = Phase.Seeking
let p2: Phase = Phase.Split(Walk(z: 2, o: 5))
print(p1 is Seeking, p2 is Seeking)
