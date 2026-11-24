// Source: CCC 2021 S3 (Lunch Concert), https://dmoj.ca/problem/ccc21s3
// Problem: N <= 2e5 friends at P_i <= 1e9, walking 1 metre per W_i <= 1000 seconds, hearing
//   from within D_i <= 1e9 metres; pick an integer spot c minimizing the total walking time
//   sum W_i * max(0, |P_i - c| - D_i), and print that minimum.
// Solution: the total is convex and piecewise linear in c with breakpoints at the range ends
//   P_i - D_i and P_i + D_i; its slope starts at -sum W_i and gains W_i at each end. Sort the
//   2N ends, stop at the first where the slope becomes >= 0 and evaluate there. O(N log N).
// original: corpus/submissions/4323533.cpp
type Friend = (p: 0..=1e9, w: 1..=1000, d: 0..=1e9)
let n: 1..=2e5 = read()
let friends: [n]Friend = read()

fn walk(c: Int) -> Int = sum(f.w * max(0, abs(f.p - c) - f.d) for f in friends)

// The total is convex in c. Its slope is -w for a friend who still has to walk
// right, and it gains w at each end of a friend's hearing range, so the best
// spot is the first range end where the slope stops being negative.
var ends: List<(at: Int, w: Int)> = []
for f in friends:
    ends.push((f.p - f.d, f.w))
    ends.push((f.p + f.d, f.w))
var slope = -sum(f.w for f in friends)
for e in ends.sorted():
    slope += e.w
    if slope >= 0:
        print(walk(e.at))
        break
