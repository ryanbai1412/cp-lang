// btoi17p2 Toll: vertex v is in block v / k, and every road goes from one block to the next.
// The cheapest a -> b cost is the min-plus product of the block-to-block matrices in between,
// so a segment tree of those matrices answers each query in O(k^3 log n).
let (k, n, m, q) = read<(1..=5, 1..=50_000, 0..=10_000, 1..=10_000)>()
domain V = 0..n
domain B = 0..=(n - 1) / k
var hop: [B]Mat<?Int> = Mat.filled(k, k, none, over: Semiring(min, +))
for (u, v, w) in read<[m](V, V, 1..=10_000)>():
    hop[B(u / k)].at(u % k, v % k) = some(w)
let st = SegTree(*, hop)
for (a, b) in read<[q](V, V)>():
    if a == b:
        print(0)
    elif a / k >= b / k:
        print(-1)
    else:
        print(st.fold(B(a / k)..B(b / k))!.get(a % k, b % k) ?? -1)
