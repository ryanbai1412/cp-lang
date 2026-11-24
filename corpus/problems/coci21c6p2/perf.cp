// Tuned: the sweep runs unchecked (sums stay below 2.5e14) with a plain Int best
// and min(a, b) hoisted.
//
// Source: COCI 2021/2022 Contest 6 (Zemljiste), https://dmoj.ca/problem/coci21c6p2
// limits: 2000 ms, 512 MB
// Problem: an r x s grid (r, s <= 500) of prices c <= 1e9, and a, b <= 1e9. A non-empty
//   subrectangle with total m costs |m - a| + |m - b|; print the smallest cost.
// Solution: the cost falls until min(a, b), is flat up to max(a, b), then rises, so with
//   positive prices, for each left end only two sums matter: the smallest >= min(a, b) and
//   the largest below it. Fix the top and bottom rows, collapse to column sums and sweep
//   with two pointers: O(r^2 s).
// original: corpus/submissions/4529998.cpp
// Bounds: r, s: 1..=500; a, b: 1..=1e9; price: 1..=1e9

let (r, s): (1..=500, 1..=500) = read()
let (a, b): (1..=1e9, 1..=1e9) = read()
let price: [r, s]1..=1e9 = read()

fn cost(m: Int) -> Int = abs(m - a) + abs(m - b)

let low = min(a, b)
var best = 1 << 62
unchecked:
    for top in 0..r:
        var column: [s]Int = 0
        for bottom in top..r:
            for c in 0..s: column[c] += price[bottom, c]
            var (right, sum) = (0, 0)
            for left in 0..s:
                while right < s and sum < low:
                    sum += column[right]
                    right += 1
                best = min(best, cost(sum))
                if right - left >= 2: best = min(best, cost(sum - column[right - 1]))
                sum -= column[left]
print(best)
