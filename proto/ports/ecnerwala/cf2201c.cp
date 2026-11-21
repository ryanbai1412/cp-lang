// Codeforces 2201 C: linear modular DP counting non-empty balanced-parenthesis
// subsequences whose prefix balance stays >= 2 at open-bracket extensions.
const MOD = 998244353
type Z = Mod<MOD>

@!multitest
let _n: Int = read()
let s: Str = read()

var (ways_any, ways_open, ways_close, ans): (Z, Z, Z, Z) = (0, 0, 0, 0)
var pref = 0
for c in s:
    ans += 1
    if c == '(':
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
