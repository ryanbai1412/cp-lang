// Source: CCO 2018 Day 2 P3, Flop Sorting, https://dmoj.ca/problem/cco18p6
// limits: 1000 ms, 1024 MB
// Problem: two permutations a and b of 1..=N (N <= 4096). A flop on l..=r swaps the minimum
//   and maximum of a_l..a_r. Print at most 3e5 flops that turn a into b.
// Solution: a flop on l..=r swaps its ends when they are its min and max, so a monotone run
//   is reversed by len/2 flops. Merge sort both a and b: merge two sorted halves by rotating
//   the left half's elements >= the median of l..r past the right half's elements below it
//   (three run reversals), then recurse on the two sides. Print a's flops, then b's in reverse
//   (each flop undoes itself). O(N log^2 N) flops.
// original: corpus/submissions/4575245.cpp
// Bounds: n: 1..=4096; p: 1..=n
let n: 1..=4096 = read()
type Val = 1..=n
type Flop = (l: Int, r: Int)  // 0-based positions

var cur: List<Val> = []
var flops: List<Flop> = []

// Reverses the run cur[lo..hi]: a monotone run's ends are its min and max at every step.
fn reverse(lo: Int, hi: Int):
    var (l, r) = (lo, hi - 1)
    while l < r:
        cur.swap(l, r)
        flops.push((l:, r:))
        l += 1
        r -= 1

// Merges the sorted runs cur[l..mid] and cur[mid..r].
fn merge(l: Int, mid: Int, r: Int):
    if l == mid or mid == r: return
    let med = cur[l..r].sorted()[(r - l) / 2]
    let m1 = (l..mid).first_true(i => cur[i] >= med) ?? mid
    let m2 = (mid..r).first_true(i => cur[i] >= med) ?? r
    reverse(m1, mid)
    reverse(mid, m2)
    reverse(m1, m2)
    merge(l, m1, m1 + (m2 - mid))
    merge(m2 - (mid - m1), m2, r)

fn sort_range(l: Int, r: Int):
    if r - l <= 1: return
    let mid = (l + r) / 2
    sort_range(l, mid)
    sort_range(mid, r)
    merge(l, mid, r)

// Reads a permutation and returns the flops that sort it into 1..=n.
fn sorting_flops() -> List<Flop>:
    let p: [n]Val = read()
    cur = p
    flops = []
    sort_range(0, n)
    flops

let there = sorting_flops()  // a
let back = sorting_flops()  // b
print(there.len + back.len)
for f in there: print(f.l + 1, f.r + 1)
for f in back.rev: print(f.l + 1, f.r + 1)
