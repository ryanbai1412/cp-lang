// Source: Codeforces 2124D (Make a Palindrome), https://codeforces.com/contest/2124/problem/D
// Problem: an array a of n <= 2e5 values and k <= n (multitest, sum n <= 2e5). An operation
//   deletes the k-th smallest of some window of length >= k. Print YES if some sequence of
//   operations leaves a palindrome (the empty array counts), else NO.
// Solution: k = 1 is always YES. With thresh the (k - 1)-th smallest value, a window's k-th
//   smallest is never below thresh, so values below thresh all stay and must already read as
//   a palindrome; larger values can always go, and copies of thresh can go while k values
//   remain. Cut a at the small values into gaps of thresh copies: a palindrome keeps
//   min(gaps[i], gaps[g - 1 - i]) of each mirrored pair; YES iff that plus the small values is
//   at least k - 1. O(n log n) for the sort.
@!multitest
use std.strings.{is_palindrome}

let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=2e5 = read()
if k == 1: return print("YES")

let thresh = a.sorted()[k - 2]
let small = a.filter(v => v < thresh).to_list()
var gaps = List.filled(small.len + 1, 0)
var before = 0
for v in a:
    if v < thresh: before += 1
    elif v == thresh: gaps[before] += 1
let kept = small.len + [min(x, y) for (x, y) in zip(gaps, List(gaps.rev))].sum()
print(yes(is_palindrome(small) and kept >= k - 1))
