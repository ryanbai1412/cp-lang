enum Sign = neg | zero | pos

var cnt: [Sign]Int = 0
cnt[Sign.neg] += 3
cnt[Sign.zero] += 1
cnt[Sign.pos] += 5
let s = Sign.pos
cnt[s] += 2
print(cnt[Sign.neg], cnt[Sign.zero], cnt[Sign.pos])
