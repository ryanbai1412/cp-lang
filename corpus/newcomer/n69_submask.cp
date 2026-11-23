let n: 1..=16 = read()
let w: [0..(1 << n)]Int = read()
var best: List<Int> = [0 for _ in 0..(1 << n)]
for mask in 0..(1 << n):
    var s = mask
    while s > 0:  // expect: missing submask_loop
        best[mask] = max(best[mask], w[s])  // expect: chmin_chmax_if
        s = (s - 1) & mask
print(best)
