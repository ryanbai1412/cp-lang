// Codeforces 2081A Math Division: https://codeforces.com/problemset/problem/2081/A
// Each op drops the lowest bit; with carry c the low part b + c is 0, 2 (forced) or 1 (coin flip).
// After n - 1 ops x is 1 + carry, so E = (n - 1) + P(carry at the end).
type Z = Mod<1e9 + 7>
let half = Z(2).inv

@!multitest
let n: 1..=1e5 = read()
let s = read<Str(len: n..=n) of "01">()
var carry: Z = 0
for i in (1..n).rev:
    carry = if s[i] == '1': carry + (1 - carry) * half else: carry * half
print(Z(n - 1) + carry)
