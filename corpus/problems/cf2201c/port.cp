// Source: Codeforces 2201C, https://codeforces.com/contest/2201/problem/C
// limits: 2000 ms, 256 MB
// Problem: multitest; a regular bracket sequence s, 2 <= n <= 3e5 (sum n <= 3e5). Count the
//   non-empty subsequences whose cyclic right shift (each chosen character takes the previous
//   chosen one's value, the first takes the last's) leaves s regular, modulo 998244353.
// Solution: one scan with the prefix balance pref and the counts ways_open / ways_close of
//   chosen prefixes ending in '(' / ')' that a later ')' can still end. A subsequence ending
//   at '(' is always valid (2^i of them); ending at ')' gives 1 + ways_open + ways_close valid
//   ones, which also join ways_close. Once pref < 2 no pending prefix ending in '(' can take a
//   later ')', so ways_open resets to 0. O(n).
// Bounds: n: 2..=300_000

const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 2..=300_000 = read()
let s: [0..n]Char = read()

var (ways_open, ways_close, ans): (Z, Z, Z) = (0, 0, 0)
var pref = 0
for (i, c) in s.items:
    if c == '(':
        ans += Z(2) ** i
        ways_open += ways_open + ways_close + 1
        pref += 1
    else:
        let add = ways_open + ways_close + 1
        ans += add
        ways_close += add
        pref -= 1
    if pref < 2: ways_open = 0
print(ans)
