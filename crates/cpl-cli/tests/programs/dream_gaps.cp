type Z = Mod<998_244_353>
type Vol = 1..=5

struct Barrels:
    count: [Vol]Int = 0
    total: Int = 0

let n: 1..=10 = read()
let target: [0..n]?Int = read(none: -1)

struct SweepAdd:
    delta: [0..=n]Z = 0
    cur: Z = 0

var b = Barrels()
b.count[3] = 7
var sw = SweepAdd()
sw.delta[1] += 5
print(b.count[1], b.count[3], sw.delta[0], sw.delta[1])

var sum = 0
for i in 0..n:
    sum += target[i].filter(x => x > 0).map(x => x * 2) ?? 0
print(sum)

domain P = 1..=4
fn is_even(p: P) -> Bool = p.label % 2 == 0
let label_x10: [P]Int = [P] p => p.label * 10
let p2: ?P = Some(P(2))
let p3: ?P = Some(P(3))
let pn: ?P = None
print(yes(p2.map(is_even) ?? false), yes(p3.map(is_even) ?? false), yes(pn.map(is_even) ?? false))
print(p2.map(label_x10) ?? -1, pn.map(label_x10) ?? -1)
print(p2.filter(is_even).map(p => p.label) ?? -1, p3.filter(is_even).map(p => p.label) ?? -1)

var rest: List<Int> = []
for v in P.all.without(P(2)):
    rest.push(v.label)
print(rest)
