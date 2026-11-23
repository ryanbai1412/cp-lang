// CCO '23 P5 — Tree Walk / K-Jump Path (today form for proto/cpc)
// original: corpus/submissions/5580402.cpp
let n: 1..=200_000 = read()
let kv: 1..=3 = read()

domain V = 0..n
var adj: [V]List<Int> = []
for _ in 0..n - 1:
    let u: Int = read()
    let v: Int = read()
    adj[V(u - 1)].push(v - 1)
    adj[V(v - 1)].push(u - 1)

var arr: [V]Int = 0
for i in 0..n:
    let wi: Int = read()
    arr[V(i)] = wi

// Root tree at 0 iteratively/recursively into child lists.
var ch: [V]List<Int> = []
fn root_dfs(u: Int, p: Int):
    for idx in 0..adj[V(u)].len:
        let v = adj[V(u)][idx]
        if v != p:
            ch[V(u)].push(v)
            root_dfs(v, u)

root_dfs(0, -1)

var out_ans: List<Int> = []

// --- kv == 1 ---
var memo1: [V]Int = -1

fn dp1(u: Int) -> Int:
    if memo1[V(u)] != -1: return memo1[V(u)]
    var best = 0
    for idx in 0..ch[V(u)].len:
        let c = dp1(ch[V(u)][idx])
        if c > best: best = c
    let res = best + arr[V(u)]
    memo1[V(u)] = res
    res

fn build1(u: Int):
    out_ans.push(u + 1)
    let rem = dp1(u) - arr[V(u)]
    if rem == 0: return
    for idx in 0..ch[V(u)].len:
        let v = ch[V(u)][idx]
        if dp1(v) == rem:
            build1(v)
            return

// --- kv == 2 ---
let neg = -1_000_000_000_000_000_000
domain S2 = 0..4 * n
var memo2: [S2]Int = -1

fn s2_idx(u: Int, me: Int, bk: Int) -> Int:
    u * 4 + me * 2 + bk

fn score_child(v: Int, kind: Int) -> Int:
    // 0: (0,0), 1: (0,1), 2: (1,0), 3: (1,1)
    if kind == 0: return dp2(v, 0, 0) - arr[V(v)]
    if kind == 1: return dp2(v, 0, 1) - arr[V(v)]
    if kind == 2: return dp2(v, 1, 0) - arr[V(v)]
    dp2(v, 1, 1) - arr[V(v)]

struct Cand:
    w: Int
    v: Int

fn topk_children(u: Int, lim: Int, kind: Int) -> List<Cand>:
    var xs: List<Cand> = []
    for idx in 0..ch[V(u)].len:
        let v = ch[V(u)][idx]
        xs.push(Cand(-score_child(v, kind), v))
    xs.sort()
    var res: List<Cand> = []
    var i = 0
    while i < xs.len and i < lim:
        res.push(Cand(-xs[i].w, xs[i].v))
        i += 1
    res

struct Res2:
    val: Int
    v1: Int
    v2: Int

fn opt2(u: Int, k1: Int, k2: Int) -> Res2:
    let t1 = topk_children(u, 2, k1)
    let t2 = topk_children(u, 2, k2)
    var bv = neg
    var b1 = -1
    var b2 = -1
    for i in 0..t1.len:
        for j in 0..t2.len:
            if t1[i].v != t2[j].v:
                let s = t1[i].w + t2[j].w
                if s > bv:
                    bv = s
                    b1 = t1[i].v
                    b2 = t2[j].v
    Res2(bv, b1, b2)

struct Res3:
    val: Int
    v1: Int
    v2: Int
    v3: Int

fn opt3(u: Int, k1: Int, k2: Int, k3: Int) -> Res3:
    let t1 = topk_children(u, 3, k1)
    let t2 = topk_children(u, 3, k2)
    let t3 = topk_children(u, 3, k3)
    var bv = neg
    var b1 = -1
    var b2 = -1
    var b3 = -1
    for i in 0..t1.len:
        for j in 0..t2.len:
            if t1[i].v != t2[j].v:
                for k in 0..t3.len:
                    if t3[k].v != t1[i].v and t3[k].v != t2[j].v:
                        let s = t1[i].w + t2[j].w + t3[k].w
                        if s > bv:
                            bv = s
                            b1 = t1[i].v
                            b2 = t2[j].v
                            b3 = t3[k].v
    Res3(bv, b1, b2, b3)

fn dp2(u: Int, me: Int, bk: Int) -> Int:
    let key = S2(s2_idx(u, me, bk))
    if memo2[key] != -1: return memo2[key]
    var tot = 0
    for idx in 0..ch[V(u)].len:
        tot += arr[V(ch[V(u)][idx])]
    var best = 0
    if bk == 0:
        for idx in 0..ch[V(u)].len:
            let c = dp2(ch[V(u)][idx], 0, 0)
            if c > best: best = c
        let r2a = opt2(u, 1, 2)
        if r2a.v1 != -1 and tot + r2a.val > best: best = tot + r2a.val
        if me == 0:
            let r3 = opt3(u, 3, 1, 2)
            if r3.v1 != -1 and tot + r3.val > best: best = tot + r3.val
            let r2b = opt2(u, 3, 0)
            if r2b.v1 != -1 and tot + r2b.val > best: best = tot + r2b.val
            for idx in 0..ch[V(u)].len:
                let v = ch[V(u)][idx]
                let c = tot - arr[V(v)] + dp2(v, 0, 0)
                if c > best: best = c
    elif me == 1:
        for idx in 0..ch[V(u)].len:
            let v = ch[V(u)][idx]
            let c = tot - arr[V(v)] + dp2(v, 0, 1)
            if c > best: best = c
    else:
        for idx in 0..ch[V(u)].len:
            let v = ch[V(u)][idx]
            let c = tot - arr[V(v)] + dp2(v, 1, 1)
            if c > best: best = c
    let res = best + arr[V(u)]
    memo2[key] = res
    res

fn emit_excl(u: Int, s1: Int, s2: Int, s3: Int):
    for idx in 0..ch[V(u)].len:
        let v = ch[V(u)][idx]
        if v != s1 and v != s2 and v != s3:
            out_ans.push(v + 1)

fn build2(u: Int, me: Int, bk: Int):
    let rem = dp2(u, me, bk) - arr[V(u)]
    if rem == 0:
        out_ans.push(u + 1)
        return
    var tot = 0
    for idx in 0..ch[V(u)].len:
        tot += arr[V(ch[V(u)][idx])]
    if bk == 0:
        for idx in 0..ch[V(u)].len:
            let v = ch[V(u)][idx]
            if dp2(v, 0, 0) == rem:
                out_ans.push(u + 1)
                build2(v, 0, 0)
                return
        let r2a = opt2(u, 1, 2)
        if r2a.v1 != -1 and tot + r2a.val == rem:
            out_ans.push(u + 1)
            build2(r2a.v1, 0, 1)
            emit_excl(u, r2a.v1, r2a.v2, -1)
            build2(r2a.v2, 1, 0)
            return
        if me == 0:
            let r3 = opt3(u, 3, 1, 2)
            if r3.v1 != -1 and tot + r3.val == rem:
                build2(r3.v1, 1, 1)
                out_ans.push(u + 1)
                build2(r3.v2, 0, 1)
                emit_excl(u, r3.v1, r3.v2, r3.v3)
                build2(r3.v3, 1, 0)
                return
            let r2b = opt2(u, 3, 0)
            if r2b.v1 != -1 and tot + r2b.val == rem:
                emit_excl(u, r2b.v1, r2b.v2, -1)
                build2(r2b.v1, 1, 1)
                out_ans.push(u + 1)
                build2(r2b.v2, 0, 0)
                return
            for idx in 0..ch[V(u)].len:
                let v = ch[V(u)][idx]
                if tot - arr[V(v)] + dp2(v, 0, 0) == rem:
                    emit_excl(u, v, -1, -1)
                    out_ans.push(u + 1)
                    build2(v, 0, 0)
                    return
    elif me == 1:
        for idx in 0..ch[V(u)].len:
            let v = ch[V(u)][idx]
            if tot - arr[V(v)] + dp2(v, 0, 1) == rem:
                out_ans.push(u + 1)
                build2(v, 0, 1)
                emit_excl(u, v, -1, -1)
                return
    else:
        for idx in 0..ch[V(u)].len:
            let v = ch[V(u)][idx]
            if tot - arr[V(v)] + dp2(v, 1, 1) == rem:
                emit_excl(u, v, -1, -1)
                build2(v, 1, 1)
                out_ans.push(u + 1)
                return

// --- kv == 3 ---
fn build3(u: Int, pre: Bool):
    if pre: out_ans.push(u + 1)
    for idx in 0..ch[V(u)].len:
        build3(ch[V(u)][idx], not pre)
    if not pre: out_ans.push(u + 1)

if kv == 1:
    print(dp1(0))
    build1(0)
elif kv == 2:
    print(dp2(0, 1, 0))
    build2(0, 1, 0)
else:
    var sum_all = 0
    for i in 0..n: sum_all += arr[V(i)]
    print(sum_all)
    build3(0, true)

print(out_ans.len)
print(out_ans)
