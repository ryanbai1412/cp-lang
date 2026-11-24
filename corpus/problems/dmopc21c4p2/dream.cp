// DMOPC 2021 C4 P2: in increasing order, keep a value unless a kept value divides it; the kept
// values are the group minimums, so count them.
let n: 1..=2e6 = read()
let size: [n]1..=5e6 = read()
let sieve = Sieve(5e6)
var kept: [0..=5e6]Bool = false
for x in size.sorted():
    if not sieve.divisors(x).any(d => kept[d]): kept[x] = true
print(kept.count(k => k))
