let n: 1..=100 = read()
domain V = 1..=n
let a: [V]Int = read()

var pre: [V.bounds]Int = 0
for v in V:
    pre[v.r] = pre[v.l] + a[v]

var best: ?Int = none
var min_pre = pre[V.first.l]
for v in V:
    best max= pre[v.r] - min_pre
    min_pre min= pre[v.r]
print(best!)

@brute(main)
fn slow():
    var ans: ?Int = none
    for l in V:
        var cur = 0
        for r in l..=V.last:
            cur += a[r]
            ans max= cur
    print(ans!)
