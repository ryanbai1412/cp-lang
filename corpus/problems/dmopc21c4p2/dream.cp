// DMOPC 2021 C4 P2: keep a value when no smaller kept value divides it; count kept values.
let xs = read [read Int]1..=5_000_000
let sieve = Sieve(5_000_000)
var kept: Set<Int> = {}
for x in xs.sorted():
    if not sieve.divisors(x).any(d => d in kept):
        kept.add(x)
print(kept.len)
