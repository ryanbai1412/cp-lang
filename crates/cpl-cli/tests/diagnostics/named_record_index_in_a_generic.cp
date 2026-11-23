domain A = 0..3
domain B = 0..2
type P = (a: A, b: B)
fn get<S>(s: S) -> Int:
    var e: [S]?Int = None
    e[s] = 7
    e[s]!
print(get((a: A(1), b: B(1))))
var d: [P]Int = 0
print(d[(B(0), A(0))])
