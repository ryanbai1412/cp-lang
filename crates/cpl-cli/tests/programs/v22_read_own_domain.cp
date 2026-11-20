// `[domain S]Char = read()`: the token read sizes S.
let s: [domain S]Char = read()
print(S.len, s[S.first], s[S.last])
print(count(s[i] == 'l' for i in S))
var seen: [S]Bool = false
for i in S: seen[i] = s[i] == '-'
print([i.to_idx for i in S if seen[i]])
let t: [domain T]Char = read()
print(T.len, [t[i] for i in T])
