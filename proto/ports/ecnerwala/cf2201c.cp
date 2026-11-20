// Codeforces 2201 C: linear modular DP counting non-empty balanced-parenthesis
// subsequences whose prefix balance stays >= 2 at open-bracket extensions.
const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let n: Int = read()
let s: Str = read()

var ways_any: Z = 0
var ways_open: Z = 0
var ways_close: Z = 0
var pref = 0
var ans: Z = 0
for i in 0..n:
    ans += 1
    if s[i] == '(':
        ans += ways_any
        ways_open += ways_open + ways_close + 1
        pref += 1
    else:
        ans += ways_open + ways_close
        ways_close += ways_open + ways_close + 1
        pref -= 1
    if pref < 2: ways_open = 0
    ways_any = ways_any * 2 + 1
print(ans)
