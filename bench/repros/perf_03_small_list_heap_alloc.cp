// Perf Repro #3 (P0 — #2 shoi16p2 12.17x, #7 ioi09p6 4.19x, #9 sac22cc2p4 4.11x,
//                #16 ucrpc21f 2.81x):
// Small fixed-size list literals (e.g. 4-neighbor directions `[(0,1),(1,0),(0,-1),(-1,0)]`)
// and small non-escaping local lists inside hot loops emit heap-allocated `std::vector`
// (`malloc`/`free` per loop iteration — 10M+ allocations on max-N grids).
// Fix: lower constant-length / small non-escaping lists to `std::array` on the stack
// (or static `constexpr std::array` for pure constant literals).

let n: Int = read()
var total = 0
for i in 0..n:
    for (dr, dc) in [(0, 1), (1, 0), (0, -1), (-1, 0)]:
        total += (i + dr) ^ (i + dc)
print(total)
