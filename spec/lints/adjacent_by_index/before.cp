let n: 2..=100 = read()
domain V = 1..=n
let a: [V]Int = read()
var cnt = 0
for v in V:
    if v != V.last!:
        if a[v] < a[v.next!]:
            cnt += 1
print(cnt)
