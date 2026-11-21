struct Best:
    d1: Int
    ch1: Int
    d2: Int

fn merge_best(a: Best, o: Best) -> Best:
    if o.ch1 == a.ch1:
        Best(max(a.d1, o.d1), a.ch1, max(a.d2, o.d2))
    elif o.d1 > a.d1:
        Best(o.d1, o.ch1, max(a.d1, o.d2))
    elif o.d1 > a.d2:
        Best(a.d1, a.ch1, o.d1)
    else:
        a

@! multitest
let n: Int = read()
let s: Str = read()
var adj: [0..n]List<Int> = []
for _ in 0..n - 1:
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)
var ans: [0..n]Int = -1
var min_dist: [0..n]Int = 0
for i in 0..n:
    if s[i] == '1':
        ans[i] = 1
        min_dist[i] = 2 * n
var round = 0
while true:
    for i in 0..n:
        min_dist[i] /= 2
    var next_dist: [0..n]Int = 0
    var alive: [0..n]Bool = true
    var sz: [0..n]Int = 0
    var bests: List<Best> = []

    fn calc_sz(cur: Int, prv: Int) -> Int:
        if not alive[cur]:
            sz[cur] = 0
            return 0
        var v = 1
        for nxt in adj[cur]:
            if nxt != prv:
                v += calc_sz(nxt, cur)
        sz[cur] = v
        v

    fn collect(cur: Int, prv: Int, depth: Int, ch: Int, tot_sz: Int):
        if min_dist[cur] > 0 and min_dist[cur] >= depth:
            let idx = min(tot_sz, min_dist[cur] - depth)
            bests[idx] = merge_best(bests[idx], Best(depth, ch, -1))
        for nxt in adj[cur]:
            if alive[nxt] and nxt != prv:
                collect(nxt, cur, depth + 1, ch, tot_sz)

    fn query_node(cur: Int, ch: Int, depth: Int):
        let b = bests[depth]
        let d = if b.ch1 == ch: b.d2 else: b.d1
        if d != -1 and s[cur] == '1':
            next_dist[cur] max= depth + d

    fn answer(cur: Int, prv: Int, depth: Int, ch: Int):
        query_node(cur, ch, depth)
        for nxt in adj[cur]:
            if alive[nxt] and nxt != prv:
                answer(nxt, cur, depth + 1, ch)

    fn rec_centroid(start: Int):
        var rt = start
        let tot_sz = calc_sz(rt, -1)
        while true:
            var found = false
            for nxt in adj[rt]:
                if alive[nxt] and sz[nxt] < sz[rt] and sz[nxt] * 2 >= tot_sz:
                    rt = nxt
                    found = true
                    break
            if not found:
                break
        bests = List.filled(tot_sz + 1, Best(-1, -1, -1))
        if min_dist[rt] > 0:
            let idx = min(tot_sz, min_dist[rt])
            bests[idx] = merge_best(bests[idx], Best(0, rt, -1))
        for ch in adj[rt]:
            if alive[ch]:
                collect(ch, rt, 1, ch, tot_sz)
        var i = tot_sz
        while i > 0:
            bests[i - 1] = merge_best(bests[i - 1], bests[i])
            i -= 1
        query_node(rt, rt, 0)
        for ch in adj[rt]:
            if alive[ch]:
                answer(ch, rt, 1, ch)
        alive[rt] = false
        for ch in adj[rt]:
            if alive[ch]:
                rec_centroid(ch)

    rec_centroid(0)
    var has = false
    for i in 0..n:
        if next_dist[i] > 0:
            ans[i] = round + 2
            has = true
    min_dist = next_dist
    if not has:
        break
    round += 1
print(ans)
