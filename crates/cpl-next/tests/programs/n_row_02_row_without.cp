// expect: ok, output "7\n2 6\n9 3\n6"
// `..r without (u, v)` is the record's other fields, of a row-bound `R` or a written record,
// after any it adds; `..R without (u, v)` is its type, through an alias too (R08.4.1).
fn strip<R: (u: Int, ..)>(r: R) -> (..R without (u)) = (..r without (u))
let s = strip((u: 1, w: 7))
print(s.w)
fn arcs<V, R: (u: V, v: V, ..)>(edges: List<R>) -> List<(to: V, ..R without (u, v))> =
    [(to: e.v, ..e without (u, v)) for e in edges]
let a = arcs([(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 6)])
print(a[1].to, a[1].w)
let c = (u: 1, v: 2, w: 3)
let d = (to: 9, ..c without (u, v))
print(d.to, d.w)
type Arc<V, R> = (to: V, ..R without (u, v))
fn first<V, R: (u: V, v: V, ..)>(e: R) -> Arc<V, R> = (to: e.v, ..e without (u, v))
print(first((u: 4, v: 5, c: 6)).c)
