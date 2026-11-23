// Source: Codeforces 2261D1 (Come Be My Friend, Don't Be So Scared, easy version),
//   https://codeforces.com/contest/2261/problem/D1
// Problem: multitest; n hiding spots in a row (n <= 2e5, sum of n <= 2e5) and m = 0 given
//   detectors. Print the fewest segment detectors [u, v] whose yes/no reports determine
//   every spot (each spot has a different set of covering segments), then the segments.
// Solution: neighbouring spots differ only across a segment end, so the n - 1 gaps need
//   2k >= n - 1 ends; for odd n, 2k = n - 1 forces no segment to touch spot 1 or n, leaving
//   both uncovered. So k >= ceil(n / 2) for n >= 2 (0 for n = 1). With half = ceil(n / 2),
//   the segments [p, p + half - 1] for p = 1..half reach it: spot i lies in those starting
//   at max(1, i - half + 1)..min(i, half), a different range for every spot. O(n).

@!multitest
let (n, _m): (1..=200_000, 0..=0) = read()
if n == 1:
    print(0)
    return
domain Pos = 1..=n
let half = ceil_div(n, 2)
print(half)
for p in Pos(1)..=Pos(half): print(p, p.offset(half - 1)!)
