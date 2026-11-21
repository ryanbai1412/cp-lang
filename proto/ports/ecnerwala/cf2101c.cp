@!multitest
let n: Int = read()
let a: [0..n]Int = read()
domain V = 0..=n

var ans = 0
var locs: List<Int> = []
for z in 0..2:
    var dsu = Dsu<V>()
    var lbl = [V] v => v.ord
    for i in 0..n:
        let ai = if z == 0: a[i] else: a[n - 1 - i]
        let x = dsu.find(V(ai))
        if lbl[x] > 0:
            if let (kept, merged) = dsu.merge(x, V(lbl[x] - 1)):
                lbl[kept] min= lbl[merged]
            if z == 0:
                locs.push(i)
            elif let last = locs.last, n - 1 - i > last:
                ans += n - 1 - i - last
                locs.pop()
            else:
                break
    locs.reverse()

print(ans)
