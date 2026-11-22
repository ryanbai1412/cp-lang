// ccc18s4 Balanced Trees: trees(w) sums trees(w / k) over k in 2..=w. w / k takes O(sqrt w)
// distinct values, so the sum runs over quotient blocks, memoized by hash (n is up to 1e9).
let n = read<1..=1_000_000_000>()
@memo(map)
fn trees(w: Int) -> Int:
    if w == 1:
        return 1
    floor_quotients(w, from: 2).map(b => (b.hi - b.lo) * trees(b.q)).sum()
print(trees(n))
