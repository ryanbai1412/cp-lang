// Perf Repro #7 (P1 — #11 ioi09p4 3.58x, #15 ccc18s4 3.00x, #19 ioi04p4 2.38x):
// `@memo` emits `std::unordered_map` per-key hashing and heap node allocation even when
// parameters are small bounded integers or domains (where a flat dense array is 3-4x faster
// and uses 2-3x less RSS).
// Fix: emit flat `std::vector` / `cpl::Grid` memo tables when `@memo` parameters are bounded
// domains/ranges, and flat open-addressed hash tables when sparse.

let n: Int = read()
@memo
fn ways(i: Int, j: Int) -> Int:
    if i == 0 or j == 0: return 1
    (ways(i - 1, j) + ways(i, j - 1)) % 1000000007

print(ways(n, n))
