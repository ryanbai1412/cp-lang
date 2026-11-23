// Source: Codeforces 2124C (Subset Multiplication), https://codeforces.com/contest/2124/problem/C
// Problem: a has a_i | a_{i+1}; b is a with some subset multiplied by an unknown x. Given b
//   (n <= 6e5, b_i <= 1e9, sum n <= 6e5, multitest), print any valid x (1 <= x <= 1e9).
// Solution: where b_i does not divide b_{i+1}, b_i was multiplied and b_{i+1} was not, so b_i / x
//   divides b_{i+1}: b_i / gcd(b_i, b_{i+1}) divides x. The lcm of those over all such adjacent
//   pairs (1 if none) is valid: O(n log 1e9).

@!multitest
let n: 2..=600_000 = read()
let b: [0..n]1..=10 ** 9 = read()
var res = 1
for (x, y) in b.adjacent if y % x != 0:
    res = lcm(res, x / gcd(x, y))
print(res)
