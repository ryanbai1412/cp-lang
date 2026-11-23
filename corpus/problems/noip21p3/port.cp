// Source: NOIP 2021 T3 (Variance), https://dmoj.ca/problem/noip21p3
// Problem: a non-decreasing sequence of n <= 1e4 values, 1 <= a[i] <= 600. Any number of times,
//   replace an inner a[i] by a[i-1] + a[i+1] - a[i]. Print n^2 times the least variance,
//   which is n * (sum of squares) - (sum)^2.
// Solution: the operation swaps the two gaps next to a[i], so the gaps can be put in any
//   order, and the best order is a valley: gaps taken from small to large go to either end.
//   With the left value at 0, rest(i, s) (memo; i gaps placed, s the sum so far) puts gap i
//   on the right, adding one value as tall as all gaps so far, or on the left, lifting the
//   i + 1 earlier values by it. Only <= 600 gaps are nonzero, so O(600 * n * 600) states.
// original: corpus/submissions/5445678.cpp

use std.prefix.{prefix_fold}

let n: 1..=10000 = read()
let a: [n]1..=600 = read()
let gaps = sorted([a[i + 1] - a[i] for i in 0..n - 1])
let height = prefix_fold(+, gaps)  // height[k]: the first k gaps together
let top = n * (a[n - 1] - a[0])

// gaps i.. are still to place around i + 1 values that sum to s: n times
// the squares they add, less the square of the final sum
@memo
fn rest(i: 0..n, s: 0..=top) -> Int:
    if i == n - 1: return -s * s
    let d = gaps[i]
    let tall = height[i + 1]
    let lift = (i + 1) * d
    min(rest(i + 1, s + tall) + n * tall * tall, rest(i + 1, s + lift) + n * (2 * d * s + lift * d))

print(rest(0, 0))
