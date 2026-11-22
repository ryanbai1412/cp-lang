// NOIP 2021 Variance: a non-decreasing sequence a. Any number of times,
// replace an inner a[i] by a[i-1] + a[i+1] - a[i]; that swaps the two gaps
// next to it, so the gaps can be put in any order. Print n^2 times the
// least variance, which is n * (sum of squares) - (sum)^2.
// The best order is a valley: gaps taken from small to large go to either
// end. With the left value at 0, a gap put on the right adds one value, as
// tall as all the gaps so far; put on the left it lifts every earlier value.
// original: corpus/submissions/5445678.cpp

let n: 1..=10000 = read()
let a: [n]1..=600 = read()
let gaps = sorted([a[i + 1] - a[i] for i in 0..n - 1])
let height = scan(gaps, +)  // height[i]: gaps 0..=i together
let top = n * (a[n - 1] - a[0])

// gaps i.. are still to place around i + 1 values that sum to s: n times
// the squares they add, less the square of the final sum
@memo
fn rest(i: 0..n, s: 0..=top) -> Int:
    if i == n - 1: return -s * s
    let d = gaps[i]
    let tall = height[i]
    let lift = (i + 1) * d
    min(rest(i + 1, s + tall) + n * tall * tall, rest(i + 1, s + lift) + n * (2 * d * s + lift * d))

print(rest(0, 0))
