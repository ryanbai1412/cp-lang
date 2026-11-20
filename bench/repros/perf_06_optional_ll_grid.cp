// Perf Repro #6 (P1 — #6 yac6p6 4.75x, #9 sac22cc2p4 4.11x, #17 cco23p2 2.65x,
//                #18 coci21c1p3 2.39x, btoi17p2 3.06x & RSS 6.08x):
// While plain `[R, C]?Int` uses 8-byte niche-packed `cpl::OptInt`, using lifted
// arithmetic (`dp[...] + 1`) or `min=` on `?Int` defeats niche packing and forces
// the ENTIRE `cpl::Grid` to emit 16-byte `std::optional<ll>` (2x memory bandwidth
// + alignment padding).
// Fix: support lifted `+` and `min=` directly on `cpl::OptInt` so niche packing is
// preserved for `Grid`, `vector`, and `struct` fields.

let (h, w, kmax): (Int, Int, Int) = read()
domain KDom = 0..=kmax
var dp: [0..h, 0..w, KDom]?Int = none
dp[0, 0, KDom(0)] = 0
for i in 0..h:
    for j in 0..w:
        var best: ?Int = dp[i, j, KDom(0)]
        if i > 0:
            best min= dp[i - 1, j, KDom(0)] + 1
        dp[i, j, KDom(0)] = best
print(dp[h - 1, 0, KDom(0)] ?? -1)
