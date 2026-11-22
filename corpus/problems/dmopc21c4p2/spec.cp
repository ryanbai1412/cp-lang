// dmopc21c4p2: keep numbers in increasing order unless one already kept divides them; count the
// kept ones. A smallest-prime-factor sieve lists each number's divisors.
let n = read<1..=1_000_000>()
let sieve = Sieve(5_000_000)
var kept: [0..=5_000_000]Bool = false
var count = 0
for a in read<[n]1..=5_000_000>().sorted():
    if not sieve.divisors(a).any(d => kept[d]):
        kept[a] = true
        count += 1
print(count)
