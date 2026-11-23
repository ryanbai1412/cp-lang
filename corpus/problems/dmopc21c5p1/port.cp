// Source: DMOPC '21 Contest 5 P1 (Permutations & Primes), https://dmoj.ca/problem/dmopc21c5p1
// Problem: given n <= 1e6, print a permutation of 1..=n in which no two neighbours sum to a
//   prime, or -1 if none exists.
// Solution: two odds or two evens sum to an even number >= 4, so list the odds, then the evens;
//   the one odd-even junction must be composite: end the odds with 5 and start the evens with
//   4 (5 + 4 = 9). n = 1 prints 1; n in 2..=4 has no answer (every odd-even sum is prime). O(n).
let n: 1..=1e6 = read()
if n == 1: print(1)
elif n <= 4: print(-1)
else:
    let odds = [x for x in (1..=n).step(2) if x != 5]
    let evens = [x for x in (2..=n).step(2) if x != 4]
    print(odds + [5, 4] + evens)
