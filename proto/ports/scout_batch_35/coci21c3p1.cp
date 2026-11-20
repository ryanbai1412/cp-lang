// COCI 2021/22 #3 Lista: a list of n numbers and k. Find a block that is
// followed by k - 1 more copies of itself; print its length and the block,
// or -1. Among all such blocks take the leftmost, then the shortest.
// original: corpus/submissions/4196130.cpp
let (n, k): (1..=100, 2..=100) = read()
let a: [0..n]Int = read()

// k copies of a[l..l + len] back to back: each element equals the one len later
fn repeats(l: Int, len: Int) -> Bool:
    all(a[i] == a[i + len] for i in l..l + (k - 1) * len)

let hit = first((l, len) for l in 0..n for len in 1..=(n - l) / k if repeats(l, len))
if let (l, len) = hit:
    print(len)
    print(a[l..l + len])
else: print(-1)
