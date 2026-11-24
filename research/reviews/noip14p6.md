# noip14p6

Score: pending
Dream: pending

Re-port for headroom: f(x) = f(x mod p) (mod p), so three primes near 10^4 each decide every x
with one pass over their residues, and only survivors get the exact Horner check modulo
2^61 - 1. That keeps the one-in-2^61 guarantee of the old port and drops `--max` from 0.91 s
to 0.06 s natively (AC 0.85 s), 4.0 MB RSS against the AC's 5.5 MB.
