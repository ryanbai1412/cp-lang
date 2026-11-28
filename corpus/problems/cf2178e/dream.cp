// Codeforces 2178 E — Flatten or Concatenate (interactive)
//
// The hidden array is constructed by concatenating equal-sum halves, so any
// active subsegment `[l..=r]` with `l < r` splits into two contiguous halves
// of exact sum `tot / 2` (hence `tot` is always even and `tot /= 2` is exact).
//
// Binary-searching `md in l..r` for the first position with `2 * ask(l, md) >= tot`
// locates the end `mid` of the left half (the fallback `?? r.prev!` is never
// needed when sums are positive, guarding the search range `l..r`). Stepping
// into the shorter of `[l..=mid]` and `[mid + 1..=r]` at least halves the
// remaining length each round, isolating the maximum element in `O(log^2 n)` queries.

let wire = Wire()

@!multitest
let n: 1..=100_000 = read()
domain Pos = 1..=n

fn ask(lo: Pos, hi: Pos) -> Int = wire.ask("? {lo} {hi}")

var l = Pos(1)
var r = Pos(n)
var tot = ask(l, r)
while l < r:
    let mid = (l..=r.prev!).first_true(md => 2 * ask(l, md) >= tot) ?? r.prev!
    tot /= 2
    if r.ord - mid.ord <= mid.ord - l.ord + 1:
        l = mid.next!
    else:
        r = mid
print("!", tot)
