// Codeforces 1950E Nearly Shortest Repeating Substring
// in: t; per test n, then a lowercase string s of length n; out: per test, the shortest length of k
use std.math.{divisors}

@!multitest
let n: 1..=2e5 = read()
let s = read<Str>()

fn mismatches(s: Str, k: Int, start: Int) -> Int =
    s.keys.count(i => s[i] != s[start + i % k])

let len = divisors(n).find(k => mismatches(s, k, 0) <= 1 or (2 * k <= n and mismatches(s, k, k) <= 1))!
print(len)
