# Friction report: dmpg18s5

No gap. The v0.2 port stored every query in four parallel `[0..q]Int` arrays to find the largest value, then built its own divisor lists for every value. The v0.3 port reads queries online, uses the statement bound `Val = 1..=2e5` as a domain, std `Sieve.divisors`, `Block`/`Pos` domains, and a `[Block, Val]0..=SIDE` counter table (162 MB at max, where an `Int` table would need 640 MB).

## Measurements

gen.py --max (N = Q = 2e5, highly composite values): port 0.96 s / 162 MB, accepted C++ 0.74 s / 99 MB.
