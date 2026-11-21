@! multitest
let n: Int = read()
domain I = 0..n
let a: [I]Int = read()
var ans = 0
var locs: List<Int> = []
for z in 0..2:
    var par: [0..=n]Int = -1
    var lbl: [0..=n]Int = 0
    for i in 0..=n:
        lbl[i] = i
    fn find(start: Int) -> Int:
        var x = start
        while par[x] >= 0:
            if par[par[x]] >= 0:
                par[x] = par[par[x]]
            x = par[x]
        x
    for i in I:
        let ai = if z == 0: a[i] else: a[I(n - 1 - i.ord)]
        let x = find(ai)
        if lbl[x] > 0:
            var u = find(lbl[x] - 1)
            var v = x
            if u != v:
                lbl[v] = lbl[u]
                if par[u] > par[v]:
                    swap(mut u, mut v)
                par[u] += par[v]
                par[v] = u
            if z == 1:
                if let last = locs.last():
                    if n - 1 - i.ord > last:
                        ans += (n - 1 - i.ord) - last
                        let _ = locs.pop()
                    else:
                        break
                else:
                    break
            else:
                locs.push(i.ord)
    locs.reverse()
print(ans)
