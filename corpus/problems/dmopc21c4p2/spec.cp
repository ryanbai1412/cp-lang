// dmopc21c4p2: keep numbers in increasing order unless one already kept divides them; count the
// kept ones. A smallest-prime-factor sieve lists each number's divisors.
let n = read<1..=2_000_000>()
const LIM = 5_000_000
let sieve = Sieve(LIM)
var kept: [0..=LIM]Bool = false
var count = 0
for a in read<[n]1..=LIM>().sorted():
    if not sieve.divisors(a).any(d => kept[d]):
        kept[a] = true
        count += 1
print(count)
