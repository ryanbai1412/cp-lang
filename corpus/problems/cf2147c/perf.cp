// Codeforces 2147 C: Rabbits, tuned: port.cp's rule, scanned as orig.cpp does,
// one run at a time with no list of runs and no `link` closure per letter.
@!multitest
let n: 1..=200_000 = read()
let s: Str = read()
let len = min(n, s.len)
var ok = true
var i = 0
while i < len:
    var j = i + 1
    while j < len and s[j] != s[j - 1]: j += 1
    if s[i] == '1' and (j - i) % 4 == 3: ok = false
    i = j
print(yes(ok))
