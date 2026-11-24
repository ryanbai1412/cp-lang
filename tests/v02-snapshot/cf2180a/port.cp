// Source: Codeforces 2180A (Carnival Wheel), https://codeforces.com/contest/2180/problem/A
// Problem: a wheel of sections 0..l-1, pointer at a; each spin moves it b forward (mod l).
//   After any number of spins, print the largest section reachable. l, b <= 5000. Multitest.
// Solution: the multiples of b mod l are exactly the multiples of g = gcd(l, b), so the
//   reachable sections are those congruent to a mod g; the largest is l - g + a % g. O(log l).

@!multitest
let (l, a, b): (1..=5_000, 0..=5_000, 1..=5_000) = read()
let g = gcd(l, b)
print(l - g + a % g)
