// cpl 0.3.0
// Problem: length of the longest strictly increasing subsequence (CSES 1145).
// Input: n, then a_1..a_n. Output: the length.
// Hand-written patience sort; `a.lis()` exists in std/seq.cp but the point
// is the binary search. Guesses: H-INT-RANGE-BSEARCH (cut type of
// `tails.keys.bsearch` is Int), H-INLINE-IF-STMT.

let n = read<1..=2e5>()
let a = read<[n]1..=1e9>()
var tails = List<Int>()
for x in a:
    let c = tails.keys.bsearch(i => tails[i] >= x)
    if c == tails.len: tails.push(x) else: tails[c] = x
print(tails.len)
