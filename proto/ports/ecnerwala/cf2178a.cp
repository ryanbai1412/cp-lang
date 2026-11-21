// Codeforces 2178 A: at most one 'Y' can remain when adjacent 'Y's cannot merge.
@!multitest
let s: Str = read()
print(if s.count('Y') <= 1: "YES" else: "NO")
