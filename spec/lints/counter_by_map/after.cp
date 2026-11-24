let a = [3, 1, 3, 2, 3]
var cnt = Counter<Int>()
for x in a:
    cnt.push(x)
print(cnt[3])
let freq = a.tally()
let odd = [c for (k, c) in freq.items if k % 2 == 1]
print(odd.len)
