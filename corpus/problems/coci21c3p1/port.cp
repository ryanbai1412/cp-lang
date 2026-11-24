// Source: COCI 2021/2022 Contest 3 #1 (Lampice), https://dmoj.ca/problem/coci21c3p1
// limits: 1000 ms, 512 MB
// Problem: n <= 50 lamp colours 1 <= a_i <= 1000 and 1 <= k <= n. Cut lamps off both ends so some
//   pattern repeats k times in a row; print the pattern's length and the pattern (any one), or -1.
// Solution: brute force: for every start and pattern length with k lengths of lamps left, check
//   a[i] == a[i + len] across the block; print the first hit. O(n^3).
// original: corpus/submissions/4196130.cpp
// Bounds: n: 1..=50; k: 1..=n; a: 1..=1000

let n: 1..=50 = read()
let k: 1..=n = read()
let a: [n]1..=1000 = read()

fn repeats(start: Int, len: Int) -> Bool = all(a[i] == a[i + len] for i in start..start + (k - 1) * len)

fn first_block() -> ?(start: Int, len: Int):
    for start in 0..n:
        for len in 1..=(n - start) / k:
            if repeats(start, len): return (start:, len:)
    None

if let b = first_block():
    print(b.len)
    print(a[b.start..b.start + b.len])
else: print(-1)
