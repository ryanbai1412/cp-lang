// NOIP 2010 Imprisoning Criminals: split the criminals over two prisons so
// that the angriest feud inside one prison is as mild as possible. Take the
// feuds from angriest down, forcing each pair apart; the first pair that is
// already forced together is the answer.
let (n, m): (2..=20000, 1..=100000) = read()
domain V = 1..=n
let feuds: [0..m](a: V, b: V, anger: 1..=1e9) = read()

var together = Dsu<V>()
var foe: [V]?V = none                   // someone known to be in the other prison
for (a, b, anger) in sorted(feuds, by: f => -f.anger):
    if together.same(a, b):
        print(anger)
        return
    if let f = foe[a]: together.unite(f, b) else: foe[a] = b
    if let f = foe[b]: together.unite(f, a) else: foe[b] = a
print(0)
