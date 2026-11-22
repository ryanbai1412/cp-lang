Hand-rolling a prime or smallest-prime-factor sieve (`j = i * i; while j <= n: is_prime[j] = false; j += i` or linear `spf[i * p] = p`) duplicates `Sieve` and `sieve_multiplicative` in `std.math`.
