// CF 1791F Range Update Point Query
// in: t; per test n q, a_1..a_n, q queries "1 l r" | "2 x". out: a_x for each "2 x", one per line.
@!multitest
let n, q: 1..=2e5 = read()
domain V = 1..=n
var a = read<[V]1..=1e9>()

enum Query:
    @tag(1) Update(l: V, r: V)
    @tag(2) Ask(x: V)

// positions whose value still changes under a digit sum (a[v] >= 10)
var live = SortedSet.of([v for v in V if a[v] >= 10])
for query in read<[q]Query>():
    match query:
        Update(l, r) =>
            for i in live.drain(after: l.prev, while: k => k <= r):
                a[i] = a[i].digits().sum()
                if a[i] >= 10: let _ = live.add(i)
        Ask(x) => print(a[x])
