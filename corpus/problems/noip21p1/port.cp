// Source: NOIP 2021 P1 (Saying Numbers), https://dmoj.ca/problem/noip21p1
// limits: 1000 ms, 512 MB
// Problem: x is skipped when it is a multiple of a number with a decimal digit 7. For each of
//   T <= 2e5 queries x <= 1e7, print -1 if x is skipped, else the next number not skipped.
// Solution: sieve the survivors up to a little past the first survivor above 1e7: each number
//   with a 7 that is still alive kills all its multiples (a dead one's multiples are dead
//   already). A query is a Bitset lookup, and the next survivor is the Bitset's `next`.
//   O(L log L) once, then a short scan per query.
// original: corpus/submissions/5445676.cpp
const LIMIT = 10000100  // past the next survivor above 1e7

let t = read<1..=2e5>()
let asks = read<[t]1..=1e7>()

domain X = 1..=LIMIT
var alive = Bitset<X>.full()
for x in X:
    if alive.contains(x) and digits(x.label).contains(7):
        for m in (x.label..=LIMIT).step(x.label): alive.remove(X(m))

for x in asks: print(if alive.contains(X(x)): alive.next(X(x + 1))!.label else: -1)
