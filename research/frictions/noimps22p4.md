# Friction Report: noimps22p4

The v0.2 port read the cards in a push loop, built the small and big prime tables with counters
(`small_cnt`, `big_cnt`, `big_id` with 0 for "none"), factored by trial division over the small
primes, wrote the sum-over-subsets transform out twice, kept per-prime tables in a `Map`
guarded by an `active_big` flag list, deduplicated query primes with a `Map<Int, Bool>`, walked
submasks with `while true` and `break`, and wrote its own `popcount`. Now the cards are one
typed read into `domain Val = 1..=2000`, `rank` maps each prime to its place in `primes`
(`?Int`, no zero sentinel), `sieve.factor` splits each value into its small mask and optional
big prime, std `sos` and `submasks` replace the hand loops, and query primes are
`sorted().deduped()`.

## Prime ranks are a hand-built table
- **Where:** `var rank: [Val]?Int` filled from `primes.items`.
- **Ideal:** `sieve.prime_index(p)`, the rank of a prime in a sieve.

## The two kinds of prime index share `Int`
- **Where:** `j < smalls` splits one rank space into mask bits and `below_big` rows.
- **Ideal:** `enum Part: Small(bit: 0..13); Big(row: Int)` from one `split(rank)`, matched once.

## Dream
```cp
let cards: [n]Val = read()
let count = cards.counts()
var below = sos([count of cards with mask m]); var below_big = [sos(...) for b in Big]
for query in queries:
    let (need, wanted) = query.split(p => part(p))
    print(sum(sign(s) * pow2[free(s)] * product(pow2[below_big[b][~s]] - 1 for b in wanted)
              for s in submasks(need)))
```

- Tokens: port 545, original 1438
- Max input (`gen.py --max`): port 0.08 s, 30492 KB, original 0.77 s, 63232 KB
