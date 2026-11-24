// Source: CCC 2021 S3 (Lunch Concert), https://dmoj.ca/problem/ccc21s3
// limits: 3000 ms, 1024 MB
// Problem: N <= 2e5 friends at P_i <= 1e9, walking 1 metre per W_i <= 1000 seconds, hearing
//   from within D_i <= 1e9 metres; pick an integer spot c minimizing the total walking time
//   sum W_i * max(0, |P_i - c| - D_i), and print that minimum.
// Solution: the total is convex and piecewise linear in c with breakpoints at the range ends
//   P_i - D_i and P_i + D_i; its slope starts at -sum W_i and gains W_i at each end, so it
//   turns non-negative at the weighted median of the 2N ends (weight W_i each). Sort the ends,
//   take the first whose prefix weight reaches sum W_i, and evaluate there. O(N log N).
// original: corpus/submissions/4323533.cpp
// Bounds: n: 1..=2e5; friends.p, friends.d: 0..=1e9; friends.w: 1..=1000
let n: 1..=2e5 = read()
let friends: [n](p: 0..=1e9, w: 1..=1000, d: 0..=1e9) = read()

fn walk(c: Int) -> Int = sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends)

let ends = [(at: f.p + side * f.d, w: f.w) for f in friends for side in [-1, 1]].sorted()
let total = sum(f.w for f in friends)
let reached = ends.map(e => e.w).collect().scan(+)
print(walk(ends[(0..ends.len).find(i => reached.prefix(i + 1) >= total)!].at))
