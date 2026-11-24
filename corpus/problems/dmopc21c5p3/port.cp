// Source: DMOPC '21 Contest 5 P3, https://dmoj.ca/problem/dmopc21c5p3
// Problem: heights of n buildings in a row form a permutation p. A token starts on height n;
//   players alternately move it from i to j with p_i > p_j and |i - j| prime, and who can't
//   move loses. Print the lexicographically smallest p where the first player wins, or -1.
//   t <= 1e5 tests, n <= 1e5, sum of n <= 5e6.
// Solution: in the identity the token moves from i to any i - prime >= 1, a subtraction
//   game; win[d] for every distance d up to the largest n (sieve, then try primes until one
//   reaches a losing d). n <= 2 is -1; if win[n - 1], print 1..n. Otherwise swap the last
//   two values, except n = 11, 36, 311, whose last three values are n, n - 2, n - 1.
// original: corpus/submissions/4218397.cpp
// win[i]: the player to move wins a pile of i when a move removes a prime.

const MAX_N = 1e5
let sieve = Sieve.new(MAX_N)

// win[i]: the player to move wins a pile of i when a move removes a prime
var win: [0..=MAX_N]Bool = false
for i in 1..=MAX_N:
    for p in sieve.primes:
        if p > i: break
        if not win[i - p]:
            win[i] = true
            break

@!multitest
let n: 1..=MAX_N = read()
if n <= 2:
    print(-1)
elif win[n - 1]:
    print((1..=n).collect())
else:
    // a losing pile: perturb the tail; three sizes need the 3-cycle (n, n - 2, n - 1)
    let tail = if n == 11 or n == 36 or n == 311: [n, n - 2, n - 1] else: [n, n - 1]
    print((1..=n - tail.len).collect() + tail)
