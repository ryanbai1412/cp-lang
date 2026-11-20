// IOI '17 P3 - Toy Train (runnable on proto/cp.py lib-run today)
let (n, m): (Int, Int) = read()
var owner: List<Int> = []
for _ in 0..n:
    let v: Int = read()
    owner.push(v)
var charge: List<Int> = []
for _ in 0..n:
    let v: Int = read()
    charge.push(v)

var rev: List<List<Int>> = []
for _ in 0..n:
    rev.push([])
var out_alive = List.filled(n, 0)
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    rev[v].push(u)
    out_alive[u] += 1

var alive = List.filled(n, true)

while true:
    // Step 1: Arezou's >= 1-step attractor of alive charging stations inside `alive`
    var good = List.filled(n, false)
    var pushed = List.filled(n, false)
    var rem = List.filled(n, 0)
    for v in 0..n:
        rem[v] = out_alive[v]
    var q: List<Int> = []
    for v in 0..n:
        if alive[v] and charge[v] == 1:
            pushed[v] = true
            q.push(v)
    var head = 0
    while head < q.len:
        let w = q[head]
        head += 1
        let preds = rev[w]
        for idx in 0..preds.len:
            let u = preds[idx]
            if not alive[u] or good[u]: continue
            if owner[u] == 1:
                good[u] = true
            else:
                rem[u] -= 1
                if rem[u] == 0: good[u] = true
            if good[u] and not pushed[u]:
                pushed[u] = true
                q.push(u)

    // Step 2: `bad = alive \ good`
    var lose = List.filled(n, false)
    var lq: List<Int> = []
    for v in 0..n:
        if alive[v] and not good[v]:
            lose[v] = true
            lq.push(v)
    if lq.len == 0: break

    // Step 3: Bita's attractor of `bad` inside `alive`
    var lhead = 0
    while lhead < lq.len:
        let w = lq[lhead]
        lhead += 1
        let preds = rev[w]
        for idx in 0..preds.len:
            let u = preds[idx]
            if not alive[u] or lose[u]: continue
            if owner[u] == 0:
                lose[u] = true
                lq.push(u)
            else:
                out_alive[u] -= 1
                if out_alive[u] == 0:
                    lose[u] = true
                    lq.push(u)

    for idx in 0..lq.len:
        let v = lq[idx]
        alive[v] = false
        let preds = rev[v]
        for j in 0..preds.len:
            let u = preds[j]
            if alive[u] and owner[u] == 0:
                out_alive[u] -= 1

var out: List<Int> = []
for v in 0..n:
    out.push(if alive[v]: 1 else: 0)
print(out)
