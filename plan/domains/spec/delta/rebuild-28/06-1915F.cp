// CF 1915F Greetings: two people meet iff one's [a, b] contains the other's. Sweep by a
// from the right, counting already-seen ends below b in a Fenwick over compressed ends.
@!multitest
let n: 1..=2e5 = read()
var ps = read<[n](a: -1e9..=1e9, b: -1e9..=1e9)>()
ps.sort(by: p => p.a)
let (domain B, ends) = compress([p.b for p in ps])
var seen = Fenwick<B, Int>()
var greetings = 0
for p in ps.rev:
    let c = ends.at(p.b)
    greetings += seen.prefix(c.l)
    seen.add(c, 1)
print(greetings)
