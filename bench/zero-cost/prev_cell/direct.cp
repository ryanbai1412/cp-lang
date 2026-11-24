// The same DP with the cell the round before stored kept by hand.
let n: 3..=5000 = read()
domain I = 0..n
let s: [I]Char = read()

var inner: [I]0..=n = 0
for i in I.rev:
    var need: [I]0..=n = 0
    var prev = 0
    for j in i.after:
        prev = if s[i] == s[j]: inner[j.prev!] else: 1 + min(inner[j], prev)
        need[j] = prev
    inner = need
print(inner[I.last!])
