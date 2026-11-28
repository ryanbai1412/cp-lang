// expect: ok, output "5\n4 6\n9"
// rule: R08.4.3
// A field read through a row bound `R: (u: V, v: V, ..)` reads the field of the record each instance is given.
fn v_of<V, R: (u: V, v: V, ..)>(e: R) -> V = e.v
fn both<V: Add, R: (u: V, v: V, ..)>(es: List<R>) -> List<V> = [e.u + e.v for e in es]
struct Edge:
    pub u: Int
    pub v: Int
fn sum_u<R: (u: Int, ..)>(rs: List<R>) -> Int:
    var t = 0
    for r in rs: t += r.u
    t
print(v_of((u: 4, v: 5, c: 6)))
print(both([(u: 1, v: 3), (u: 2, v: 4)]))
print(sum_u([(u: 4, w: 0), (u: 5, w: 1)]))
