// Source: COCI 2021/2022 Contest 3 #1 (Lampice), https://dmoj.ca/problem/coci21c3p1
// limits: 1000 ms, 512 MB
// Problem: n <= 50 lamp colours 1 <= a_i <= 1000 and 1 <= k <= n. Cut lamps off both ends so some
//   pattern repeats k times in a row; print the pattern's length and the pattern (any one), or -1.
// Solution: brute force: for every start l and pattern length len with k len lamps left, check
//   a[i] == a[i + len] across the block; print the first hit. O(n^3).
// original: corpus/submissions/4196130.cpp
let n: Int = read()
let k: Int = read()
let a: [n]Int = read()

fn repeats(l: Int, len: Int) -> Bool:
    for i in l..l + (k - 1) * len:
        if a[i] != a[i + len]: return false
    true

fn find() -> ?(Int, Int):
    for l in 0..n:
        for len in 1..=(n - l) / k:
            if repeats(l, len): return (l, len)
    None

if let hit = find():
    print(hit.1)
    print([a[i] for i in hit.0..hit.0 + hit.1])
else: print(-1)
