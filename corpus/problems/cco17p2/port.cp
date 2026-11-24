// Source: CCO 2017 Day 1 P2, Cartesian Conquest, https://dmoj.ca/problem/cco17p2
// Problem: districts are integer k x 2k rectangles added one at a time, the empire always a
//   rectangle. For an N x M empire (N, M <= 1e8), print the fewest and the most districts.
// Solution: the last district is a full strip along one side, so dp(n, m) peels k strips n x jump
//   (jump = n / 2 or 2n) off the m side and recurses on the (m - k jump) x n rest. The rest must
//   be <= 2n to peel again, so only a few k are tried; four n x n/2 strips may also be one
//   n x 2n district. Euclid-like depth, memoised on (n, m); an impossible split is None, and the answer
//   joins dp(N, M) and dp(M, N).
// original: corpus/submissions/6416978.cpp
// Bounds: n, m: 1..=1e8
let n: 1..=1e8 = read()
let m: 1..=1e8 = read()

type Count = (fewest: Int, most: Int)

fn join(a: ?Count, b: ?Count) -> ?Count:
    if let x = a:
        if let y = b: Some((fewest: min(x.fewest, y.fewest), most: max(x.most, y.most))) else: a
    else: b

// The districts of an n x m rectangle whose last strips run along its m side.
@memo
fn districts(n: Int, m: Int) -> ?Count:
    if n == 0 or m == 0: return Some((fewest: 0, most: 0))
    let jump = if n % 2 == 0: n / 2 else: 2 * n
    var best: ?Count = None
    for k in max(1, (m - 2 * n) / jump - 1)..=m / jump:
        if let rest = districts(m - k * jump, n):
            best = join(best, Some((fewest: rest.fewest + k, most: rest.most + k)))
            if n % 2 == 0: best = join(best, Some((fewest: rest.fewest + k / 4 + k % 4, most: rest.most)))
    best

let both = join(districts(n, m), districts(m, n))!
print(both.fewest, both.most)
