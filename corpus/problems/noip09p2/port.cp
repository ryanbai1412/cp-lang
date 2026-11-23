// Source: NOIP 2009 P2 (Hankson's Interesting Problem), https://dmoj.ca/problem/noip09p2
// Problem: n <= 2000 queries a0, a1, b0, b1 <= 2e9 (a1 divides a0, b0 divides b1): print how
//   many positive x have gcd(x, a0) = a1 and lcm(x, b0) = b1.
// Solution: lcm(x, b0) = b1 forces x to divide b1, so test each divisor of b1 (at most 1536)
//   directly. `divisors` is trial division, O(sqrt b1) per query: about 9e7 steps in all.
use std.math.{divisors}

@!multitest
let (a0, a1, b0, b1): (1..=2e9, 1..=2e9, 1..=2e9, 1..=2e9) = read()
print(count(gcd(x, a0) == a1 and lcm(x, b0) == b1 for x in divisors(b1)))
