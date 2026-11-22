// Codeforces 2180 B — Ashmal
//
// Append or prepend each incoming string `a` (`n <= 1_000`, total length
// `L <= 4_000`) to minimize the final string. For equal-length strings `s1 < s2`,
// both `s1 + a < s2 + a` and `a + s1 < a + s2`, so keeping the lexicographically
// smallest `s = min(s + a, a + s)` at each step is globally optimal.
// Complexity: O(L^2) character copies (`<= 1.6e7` at `L = 4_000`).

@!multitest
let n: 1..=1_000 = read()
var s = ""
for _ in 0..n:
    let a: Str = read()
    s = min(s + a, a + s)
print(s)
