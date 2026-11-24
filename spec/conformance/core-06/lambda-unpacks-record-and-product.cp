// expect: ok, output "YES\nNO\n2\n3"
// rule: 06.1.4
// `(a, b, ..) => ..` passed where a one-argument fn of a record or of a
// product-domain position is expected unpacks it, as it does a tuple.
fn holds<T>(xs: List<T>, p: T -> Bool) -> Bool:
    for x in xs:
        if not p(x): return false
    true
fn tally<D>(p: D -> Bool) -> Int:
    var n = 0
    for x in D:
        if p(x): n += 1
    n
let cells = [(r: 1, c: 2, v: 3), (r: 2, c: 3, v: 1)]
print(yes(holds(cells, (r, c, v) => r < c)))
print(yes(holds(cells, (r, c, v) => v > 2)))
domain V = [2]
var w: [V, V]Int = 0
w[0, 1] = 1
w[1, 0] = 1
print(tally<(V, V)>((u, v) => w[u, v] == 1))
print(tally<(V, V)>((u, v) => u <= v))
