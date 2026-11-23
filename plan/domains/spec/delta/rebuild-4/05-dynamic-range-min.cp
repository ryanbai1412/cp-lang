// CSES 1649 Dynamic Range Minimum Queries
// in: n q, the array, then q queries "1 k u" (set x[k] = u) or "2 a b" (min of x[a..=b]); out: one line per min query
let (n, q) = read<(Int, Int)>()
domain Pos = 1..=n
enum Query:
    @tag(1) Update(k: Pos, u: Int)
    @tag(2) Ask(a: Pos, b: Pos)
var st = SegTree(min, read<[Pos]Int>())
for _ in 0..q:
    match read<Query>():
        Update(k, u) => st.set(k, u)
        Ask(a, b) => print(st.fold(a..=b))
