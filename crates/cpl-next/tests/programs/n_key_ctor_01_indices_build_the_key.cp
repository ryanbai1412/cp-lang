// expect: ok, output "5 5 YES 2 7"
// rule: R04.2.12
// On an array keyed by a struct, `a[x1, ..., xn]` is `a[K(x1, ..., xn)]`, K's constructor call
// (decision 409): `Pair`'s ordering `new` makes `d[u, v]` and `d[v, u]` one cell, and a plain
// struct's fields take the indices in order.
struct Cell(i: Int, j: Int)
impl Index for Cell:
    let count = 9
    @get fn ord(self) -> Int = i * 3 + j
    fn nth(k: Int) -> Cell = Cell(k / 3, k % 3)
domain V = 0..4
var d = [Pair<V>] p => 0
d[V(1), V(3)] = 5
var c: [Cell]Int = 0
c[0, 2] = 2
c[2, 1] += 7
print(d[V(1), V(3)], d[V(3), V(1)], if d[V(3), V(1)] == d[Pair<V>(V(1), V(3))]: "YES" else: "NO", c[Cell(0, 2)], c[2, 1])
