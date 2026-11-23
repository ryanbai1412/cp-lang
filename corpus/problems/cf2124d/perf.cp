// Codeforces 2124 D, tuned: port.cp with the threshold found by `select`
// (expected O(n)), as orig.cpp does with nth_element, instead of a full sort.
@!multitest
use std.strings.{is_palindrome}

let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=2e5 = read()
if k == 1: return print("YES")

var order = a.copy()
order.select(k - 2)
let thresh = order[k - 2]
let small = a.filter(v => v < thresh).to_list()
var gaps = List.filled(small.len + 1, 0)
var before = 0
for v in a:
    if v < thresh: before += 1
    elif v == thresh: gaps[before] += 1
let kept = small.len + [min(x, y) for (x, y) in zip(gaps, gaps.rev)].sum()
print(yes(is_palindrome(small) and kept >= k - 1))
