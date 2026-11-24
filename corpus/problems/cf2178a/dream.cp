// Codeforces 2178 A — Yes or Yes (`2 <= s.len <= 100`)
//
// Merging `'N' + 'N' -> 'N'` and `'Y' + 'N' -> 'Y'` (in either order) preserves
// the total count of `'Y'`s, while `'Y' + 'Y'` cannot be merged. Therefore `s`
// can be reduced to length 1 iff `s.count(c => c == 'Y') <= 1`.

@!multitest
let s: Str = read()
print(if s.count(c => c == 'Y') <= 1: "YES" else: "NO")
