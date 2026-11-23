// CF 1898E Sofia and Strings
// Take each letter of t from its earliest unused copy in s; every unused
// smaller letter before that copy can never pass it, so it is deleted.
domain L = 0..26

fn letter(c: Char) -> L = L(Int(c) - Int('a'))

@!multitest
let (n, m): (1..=2e5, 1..=n) = read()
let s = read<Str>()
let t = read<Str>()
var pos = [L] _ => List<Int>()
for i in 0..n: pos[letter(s[i])].push(i)
var at: [L]Int = 0
var ok = true
for ch in t:
    let c = letter(ch)
    if at[c] == pos[c].len:
        ok = false
        break
    let p = pos[c][at[c]]
    at[c] += 1
    for d in L if d < c:
        while at[d] < pos[d].len and pos[d][at[d]] < p: at[d] += 1
print(yes(ok))
