// `cur += k` on a domain value is a checked step: it stops the program when
// it leaves the domain.
let n = 5
domain I = 0..n
var cur = I(0)
var cnt: [I]Int = 0
var nxt: [I]I = [I] i => i
let k: Int = read()
cnt[cur] += 1
cur = I.at(cur.ord + 2)!
cnt[cur] += 1
nxt[cur] = nxt[cur].next!
print(cnt, cur.ord, [x.ord for x in nxt])
cur = I.at(cur.ord + k)!
print("never printed")
