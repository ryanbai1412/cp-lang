// IOI 2000 Palindrome: the fewest characters to insert into a string to make
// it a palindrome. need(l, r) is the answer for s[l..=r]: matching ends cost
// nothing, otherwise mirror one of them.
let n: 3..=5000 = read()
let s: [0..n]Char = read()

var inner: [0..n]Int = 0                   // need(l + 1, ..)
for l in (0..n).rev:
    var need: [0..n]Int = 0
    for r in l + 1..n:
        need[r] = if s[l] == s[r]: inner[r - 1] else: 1 + min(inner[r], need[r - 1])
    inner = need
print(inner[n - 1])
