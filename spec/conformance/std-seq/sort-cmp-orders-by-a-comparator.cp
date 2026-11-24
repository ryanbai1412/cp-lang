// expect: ok, output "1 2 3\n9 7 5 1\n2 1 0\n0 1 2\nbaz"
// `sort(cmp:)` and `sorted(cmp:)` (decision 87): an exchange-argument order no single key gives,
// through a method, a generic `IndexedMut` fn and the free forms; stable on ties.
struct Job: t: Int, w: Int
var js = [Job(3, 1), Job(1, 2), Job(2, 2)]
js.sort(cmp: (p, q) => p.t * q.w < q.t * p.w)
print(js.map(j => j.t).collect())
fn down<S: IndexedMut>(mut s: S) = sort(mut s, cmp: (a, b) => a > b)
var c = [5, 9, 1, 7]
down(mut c)
print(c)
var d = [(2, 1), (1, 2), (0, 5)]
sort(mut d, cmp: (p, q) => p.0 > q.0)
print(d.map(p => p.0).collect())
print(sorted(d, cmp: (p, q) => p.0 < q.0).map(p => p.0).collect())
print([(1, 'b'), (0, 'z'), (1, 'a')].sorted(cmp: (p, q) => p.0 > q.0).map(p => p.1).collect())
