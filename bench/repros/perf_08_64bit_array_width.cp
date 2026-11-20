// Perf Repro #8 (P1 — #13 coci21c1p2 3.28x & RSS #5 6.85x = 145 MB vs 21 MB;
//                also RSS #3 ccc14s4 8.93x, RSS #10 noip21p1 5.29x, cco22p4 7.37x & RSS 9.24x):
// Bounded integer arrays and 3-state DP memos default to 64-bit `std::vector<ll>`
// (8 bytes/entry vs 1-byte `int8_t` or 4-byte `int32_t`), causing 4x-8x RSS bloat
// and L2/L3 cache thrashing on large 2D/3D DP tables.
// Fix: narrow storage width for proven small ranges (e.g. `-1..=1` -> `int8_t`,
// `0..=1e9` -> `int32_t`) or offer compact `I32`/`I8` array storage.

let n: Int = read()
var memo: [0..n * n]Int = -1
for i in 0..n:
    for j in 0..n:
        memo[i * n + j] = (i + j) % 2
var s = 0
for i in 0..n * n:
    s += memo[i]
print(s)
