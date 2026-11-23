// Codeforces 2104E Unpleasant Strings: https://codeforces.com/problemset/problem/2104/E
// Match t greedily (each letter at its next occurrence). From cut c of s, the fewest letters that
// cannot all be matched is need[c] = 1 + min over letters of need[after its next occurrence],
// where a letter with no next occurrence costs 0 more.
let (n, k): (1..=1e6, 1..=26) = read()
domain I = 0..n
domain Letter = 'a'..='z'
let s = read<[I]Char>()
let allowed = Letter.take(k)
var nxt: [I.cuts, Letter]?I = None
for i in I.rev:
    for l in allowed: nxt[i.l, l] = nxt[i.r, l]
    nxt[i.l, Letter(s[i])] = i
var need: [I.cuts]Int = 0
for c in I.cuts.rev:
    need[c] = 1 + allowed.map(l => nxt[c, l].map(j => need[j.r]) ?? 0).min()!
let q: 1..=2e5 = read()
repeat q:
    let t = read<Str>()
    var cur = I.cuts.first!
    for ch in t:
        let j = nxt[cur, Letter(ch)] else:
            print(0)
            break
        cur = j.r
    else:
        print(need[cur])
