// Source: Codeforces 2089 D, Conditional Operators, https://codeforces.com/contest/2089/problem/D
// limits: 2000 ms, 250 MB
// Problem: multitest (t <= 1e4, sum n <= 1.5e5): a binary string of length 2n+1. Insert n
//   right-associative ?: operators (and any parentheses) between the characters, keeping
//   their order, so the expression evaluates to 1: print Yes and such an expression of
//   length <= 10n+1000, or No.
// Solution: Constructive, with chain(lo, hi) the flat right-associated a?b:c?d:... of a span.
//   1. length >= 5 and s = "10...1": (1?0:s2) is 0 and selects the final 1.
//   2. s[0] = 0 and s[last] = 1: 0?chain(...):1.
//   3. Otherwise some '1' at even i must be followed by a '1' at odd j > i: leading 0s skip
//      to s[i], whose true branch is (s[i+1]?chain:s[j]) = 1; no such pair means No. O(n).
// Bounds: half: 1..=200_000

@!multitest
let half: 1..=200_000 = read()
let n = 2 * half + 1
let s: Str = read()

fn chain(lo: Int, hi: Int) -> Str:
    var e = "{s[lo]}"
    for i in lo + 1..hi by 2:
        e += "?{s[i]}:{s[i + 1]}"
    e

fn first_one(from: Int) -> ?Int:
    for k in from..n by 2 if s[k] == '1':
        return k
    None

fn build() -> ?Str:
    if n >= 5 and s[0] == '1' and s[1] == '0' and s[n - 1] == '1':
        return "({s[0]}?{s[1]}:{s[2]})?{chain(3, n - 1)}:{s[n - 1]}"
    if s[0] == '0' and s[n - 1] == '1':
        return "{s[0]}?{chain(1, n - 1)}:{s[n - 1]}"
    let i = first_one(0) ?? return None
    let j = first_one(i + 1) ?? return None
    var e = if i > 0: "{s[0]}?{chain(1, i)}:" else: ""
    e += "{s[i]}?"
    e += if j > i + 1: "({s[i + 1]}?{chain(i + 2, j)}:{s[j]})" else: "{s[j]}"
    e + ":{chain(j + 1, n)}"

if let e = build():
    print("Yes")
    print(e)
else:
    print("No")
