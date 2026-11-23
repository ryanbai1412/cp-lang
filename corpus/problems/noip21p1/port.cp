// Source: NOIP 2021 P1 (Saying Numbers), https://dmoj.ca/problem/noip21p1
// Problem: x is skipped when it is a multiple of a number with a decimal digit 7. For each of
//   T <= 2e5 queries x <= 1e7, print -1 if x is skipped, else the next number not skipped.
// Solution: a sieve up to 10000100 (past the survivor after 1e7): each number with a 7 that is
//   not yet skipped marks all its multiples (a skipped one's multiples are marked already).
//   A right-to-left scan then stores the next survivor after each x. O(L log L) once, O(1) per
//   query.
// The original is its author's second attempt; the first (factor every x, test every divisor)
// is still in the file.
// original: corpus/submissions/5445676.cpp
const LIMIT = 10000100  // past the next survivor above 1e7

let t: 1..=2e5 = read()
let asks: [0..t]1..=1e7 = read()

fn has7(x: Int) -> Bool:
    digits(x).contains(7)

var skipped: [0..=LIMIT]Bool = false
for i in 1..=LIMIT:
    if not skipped[i] and has7(i):
        var j = i
        while j <= LIMIT:
            skipped[j] = true
            j += i

// after[x]: the smallest survivor above x
var after: [0..=LIMIT]Int = 0
var seen = 0
for x in (1..=LIMIT).rev:
    after[x] = seen
    if not skipped[x]: seen = x

for x in asks: print(if skipped[x]: -1 else: after[x])
