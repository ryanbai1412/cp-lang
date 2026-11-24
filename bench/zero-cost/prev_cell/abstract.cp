// ioi00p1's row DP: `need[j]` reads `need[j.prev!]`, the cell the round
// before stored, so codegen keeps that cell in a local (carry.rs).
let n: 3..=5000 = read()
domain I = 0..n
let s: [I]Char = read()

var inner: [I]0..=n = 0
for i in I.rev:
    var need: [I]0..=n = 0
    for j in i.after:
        need[j] = if s[i] == s[j]: inner[j.prev!] else: 1 + min(inner[j], need[j.prev!])
    inner = need
print(inner[I.last!])
