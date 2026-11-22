// `cur += k` on a domain value is a checked step: it stops the program when
// it leaves the domain.
let n = 5
domain I = 0..n
var cur = I(0)
var cnt: [I]Int = 0
var nxt: [I]I = [I] i => i
let k: Int = read()
cnt[cur] += 1
cur += 2
cnt[cur] += 1
nxt[cur] += 1
print(cnt, cur, nxt)
cur += k
print("never printed")
