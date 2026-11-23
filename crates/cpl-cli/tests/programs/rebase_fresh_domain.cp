let s = [5, 6, 7, 8, 9]
let w = s[1..4]
print(w.rebase[0], w.rebase[2])
let t = s[2..].rebase
print(t[t.Idx(0)], t[t.Idx.wrap(-1)], t.Idx.count)
var total = 0
for k in t.keys:
    total += t[k]
print(total)
