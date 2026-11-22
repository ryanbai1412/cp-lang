// cp: 0.1 bea787c+dirty
type E = (u: Int, w: Int)
var es: List<E> = [(1, 5), (2, 3), (3, 3)]
sort(mut es, by: e => (e.w, -e.u))
print(es[0].u)
