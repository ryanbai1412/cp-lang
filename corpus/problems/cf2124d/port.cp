// Codeforces 2124 D: Make a Palindrome.
//
// An operation deletes the k-th smallest of a window of length >= k. Let
// `thresh` be the (k - 1)-th smallest value. Fewer than k - 1 values lie below
// it, so a window's k-th smallest is never one of them: every `small` value
// stays, in order, and must already read as a palindrome. Values above `thresh`
// can always go, and copies of `thresh` can go while at least k values remain,
// so the question is whether a palindrome keeps `small` and at least k - 1
// values in all.
//
// Cut the array at the small values: `gaps[i]` copies of `thresh` sit between
// small i - 1 and small i. A palindrome keeps min(gaps[i], gaps[g - 1 - i]) of
// each mirrored pair of gaps on both sides, and the whole middle gap; summing
// min over `gaps` zipped with its reverse counts exactly that.
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
let kept = small.len + [min(x, y) for (x, y) in zip(gaps, gaps.rev)].sum()
print(if is_palindrome(small) and kept >= k - 1: "YES" else: "NO")
