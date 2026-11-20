// DMOPC '21 Contest 5 P3 — prime-step game; print a permutation the first
// player wins on, or -1.
// original: corpus/submissions/4218397.cpp
const LIMIT = 100_000
let primes = sieve(LIMIT).primes

// win(i): the player to move wins from a pile of i, removing a prime each turn.
@memo
fn win(i: 0..=LIMIT) -> Bool = any(not win(i - p) for p in primes.take_while(p => p <= i))

@multitest
let n: 1..=LIMIT = read()
if n <= 2:
    print(-1)
elif win(n - 1):
    print(1..=n)
else:
    // Losing pile: perturb the tail. Three sizes need a 3-cycle; a swap fixes the rest.
    let tail = if n in [11, 36, 311]: [n, n - 2, n - 1] else: [n, n - 1]
    print([i for i in 1..=(n - tail.len)] + tail)
