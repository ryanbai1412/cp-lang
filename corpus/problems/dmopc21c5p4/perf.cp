// Tuned: a sieve of Eratosthenes on a Bitset instead of `Sieve`, which keeps a smallest-factor
//   table four bytes per number.
//
// Source: DMOPC '21 Contest 5 P4 (Permutations & Primes), https://dmoj.ca/problem/dmopc21c5p4
// Problem: given N, print a permutation of 1..N whose adjacent gaps |p[i+1] - p[i]| are all
//   prime and whose signed differences p[i+1] - p[i] take as few distinct values as possible,
//   or -1 if none exists. Up to T tests, T, N <= 1e6, sum of N <= 1e6.
// Solution: one difference can never work, so aim for two: +a and -b with primes a < b.
//   Take a + b = N when N is even (not 6) or N is 5 or 7, else a + b = N - 1 (Goldbach;
//   scan the sieve's primes for a). Walk from 1, stepping +a while it stays <= N, else -b;
//   as gcd(a, b) = 1 this visits every value once. N = 1 prints 1, N = 2, 3 print -1, and
//   N = 4 is 3 1 4 2. One sieve to 1e6, then O(N) per test plus the prime scan.
// original: corpus/submissions/4223525.cpp
// Bounds: n: 1..=1e6

domain Num = 0..=1e6
var composite = Bitset<Num>.new()
var primes: List<Int> = []
for i in 2..=1e6:
    if not composite.contains(Num(i)):
        primes.push(i)
        for j in (i * i..=1e6).step(i): let _ = composite.add(Num(j))
fn is_prime(x: Int) -> Bool = x >= 2 and not composite.contains(Num(x))

@!multitest
let n: 1..=1e6 = read()
if n == 1:
    print(1)
elif n <= 3:
    print(-1)
elif n == 4:
    print(3, 1, 4, 2)
else:
    let total = if n == 5 or n == 7 or (n % 2 == 0 and n != 6): n else: n - 1
    let up = primes.find(a => total - a > a and is_prime(total - a))!
    let down = total - up
    var at = 1
    var walk: List<Int> = []
    for _ in 0..n:
        walk.push(at)
        at = if at + up <= n: at + up else: at - down
    print(walk)
