// Tuned: the accepted C++'s segment trees with range adds that stay on the covering nodes,
// as flat bottom-up trees in plain lists in place of a std SegTree over an Action.
//
// Source: CCO 2024 Day 2 (Heavy Light Decomposition), https://dmoj.ca/problem/cco24p5
// limits: 4000 ms, 1024 MB
// Problem: an array a of N <= 5e5 values in 1..N. A value is heavy in an array if it occurs
//   more than once there, else light; an array is good if light and heavy elements alternate.
//   Count the ways to cut a into good subarrays, mod 1000003.
// Solution: dp[i] = sum of dp[j] over ends j > i with a[i..j) good, filled from the right. The
//   first element is light or heavy, so a good array follows one of two parity patterns, one
//   SegTree of MinCount each over ends j, with range add: it keeps the least number of violated
//   conditions and the sum of dp[j] where that least is reached. Prepending a[i] adds or removes violations on ranges
//   of ends from the next occurrences of its value: a light one must not repeat, a heavy one
//   needs a later copy at a heavy place. dp[i] adds the sums where 0 are violated. O(N log N).
// Bounds: n: 1..=5e5; a: 1..=n
// original: corpus/submissions/6435483.cpp
let mod = 1_000_003
let n: 1..=5e5 = read()
domain I = 0..n
domain Value = 1..=n
let a: [I]Value = read()

// a bottom-up segment tree per side over ends 0..=n, leaf e at s * w + half + e: each node
// keeps the least violation count under it, the dp sum reaching it, and the pending add that
// covers its whole range (never pushed down)
let half = 1 << 19
let w = 2 * half
type Count = -1_000_000..=1_000_000  // a violation count or pending add, stored narrow
type Ways = 0..1_000_003
var (low, pend): (List<Count>, List<Count>) = (List.filled(2 * w, 0), List.filled(2 * w, 0))
var sum: List<Ways> = List.filled(2 * w, 0)

fn pull(t: Int, o: Int):
    let (k, l, r) = (t + o, t + 2 * o, t + 2 * o + 1)
    if low[l] == low[r]:
        let both = sum[l] + sum[r]
        (low[k], sum[k]) = (low[l] + pend[k], if both >= mod: both - mod else: both)
    elif low[l] < low[r]:
        (low[k], sum[k]) = (low[l] + pend[k], sum[l])
    else:
        (low[k], sum[k]) = (low[r] + pend[k], sum[r])

// recompute the ancestors of leaves a and b, the shared ones once
fn pull_up(t: Int, a: Int, b: Int):
    var (x, y) = (a / 2, b / 2)
    while x > 0:
        pull(t, x)
        if y != x: pull(t, y)
        (x, y) = (x / 2, y / 2)

fn put(t: Int, at: Int, v: Int):
    sum[t + half + at] = v
    pull_up(t, half + at, half + at)

fn lift(k: Int, d: Int):
    pend[k] += d
    low[k] += d

// add d to ends l..=r
fn shift(t: Int, l: Int, r: Int, d: Int):
    if l > r: return
    var (x, y) = (half + l, half + r + 1)
    while x < y:
        if x % 2 == 1:
            lift(t + x, d)
            x += 1
        if y % 2 == 1:
            y -= 1
            lift(t + y, d)
        (x, y) = (x / 2, y / 2)
    pull_up(t, half + l, half + r)

var dp = List.filled(n + 1, 0)
dp[n] = 1
var next_at = List.filled(2 * n, -1)  // [side * n + value]
var next_any = List.filled(n, -1)
var stop_of = List.filled(2 * n, n)  // [side * n + i]

for i in I.rev:
    let (k, x) = (i.ord, a[i].ord)
    for s in 0..2:
        let t = s * w
        put(t, k + 1, dp[k + 1])
        let u = next_at[(1 - s) * n + x]
        if u >= 0: shift(t, u + 1, n, 1)
        if k % 2 != s:
            let g = next_any[x]
            if g >= 0: shift(t, g + 1, n, 1)
        else:
            let d = next_at[s * n + x]
            var stop = n
            if d >= 0:
                shift(t, d + 1, stop_of[s * n + d], -1)
                stop = d
            stop_of[s * n + k] = stop
            shift(t, k + 1, stop, 1)
        if low[t + 1] == 0:
            dp[k] += sum[t + 1]
            if dp[k] >= mod: dp[k] -= mod
    next_at[k % 2 * n + x] = k
    next_any[x] = k

print(dp[0])
