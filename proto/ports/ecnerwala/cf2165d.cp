@!multitest
let n: Int = read()
let vals: [0..n]Int = read()
var idx: [0..=(2 * n)]List<Int> = []
for i in 0..n:
    idx[vals[i]].push(i)

var ans = n
for iter in 0..2:
    var cur: List<Int> = []
    for v in 0..=(2 * n):
        let cnd = if (v & 1) == iter: [n - 1 - x for x in idx[v].rev] else: idx[v]
        let prev = [n - 1 - x for x in cur.rev]
        var z = 0
        var stk = 0
        var nxt: List<Int> = []
        for a in cnd:
            while z < prev.len and prev[z] < a:
                z += 1
                stk += 1
            if stk > 0:
                stk -= 1
                ans -= 1
            else:
                nxt.push(a)
        cur = nxt
print(ans)
