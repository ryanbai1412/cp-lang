// CF 1915F, revised: print how many greetings each person takes part in, in input order.
// Person i greets everyone whose interval it contains and everyone whose interval contains it.
@!multitest
let n: 1..=2e5 = read()
let ps = read<[n](a: -1e9..=1e9, b: -1e9..=1e9)>()
let order = ps.keys.sorted(by: i => ps[i].a)
let (domain B, ends) = compress([p.b for p in ps])
var greetings: [n]Int = 0
var seen = Fenwick<B, Int>()
for i in order.rev:
    let c = ends.at(ps[i].b)
    greetings[i] += seen.prefix(c.l)
    seen.add(c, 1)
seen.clear()
for i in order:
    let c = ends.at(ps[i].b)
    greetings[i] += seen.suffix(c.r)
    seen.add(c, 1)
print(greetings)
