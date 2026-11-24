// expect: ok, output "5 5"
// rule: 08.3
// Inside a body, a bound `S: Indexed<E, R>` gives `S`'s keys the type `E` and its elements
// the type `R`, so a record spread of an element keeps `R`'s fields.
type Arc<E, R> = (to: Int, e: E, ..R without (u, v))
fn of<S: Indexed<E, R>, E, R: (u: Int, v: Int, ..)>(edges: S, keep: Arc<E, R> -> Int) -> Int:
    var t = 0
    for e in edges.keys:
        let r = edges[e]
        t += keep((to: r.v, e: e, ..r without (u, v)))
    t
fn firsts<S: Indexed<E, R>, E, R: (u: Int, v: Int, ..)>(edges: S, keep: (e: E, ..R without (u, v)) -> Int) -> Int:
    let e0 = edges.keys.first!
    let f0 = edges.at(e0)
    keep((e: e0, ..f0 without (u, v)))
print(of([(u: 3, v: 5)], keep: a => a.to), firsts([(u: 1, v: 2, w: 5)], r => r.w))
