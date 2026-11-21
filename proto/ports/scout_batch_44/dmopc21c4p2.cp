// DMOPC '21 Contest 4 P2 — greedy divisor filter: sweep the values ascending
// and keep x unless one of its divisors was already kept.
// original: corpus/submissions/4125290.cpp
let n: Int = read()
domain I = 0..n
let a: [I]Int = read()
let sv = sieve(a.max())

var kept = HashSet<Int>()
for x in a.sorted():
    // FRICTION F6: the sheet lists `sieve(n)` and `factor(x)` but no divisor walk.
    if not any(d in kept for d in sv.divisors(x)): kept.add(x)
print(kept.len)
