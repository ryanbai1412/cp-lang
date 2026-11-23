// expect: ok, output "YES"
domain V = 0..40
var j = JumpTree<V>()
var par: [V]?V = [V] _ => None
for v in 1..40:
    if v % 13 != 0:
        let p = V((v * 7 + 3) % v)
        j.add(V(v), p)
        par[V(v)] = Some(p)
fn chain(par: [V]?V, v: V) -> List<V>:
    var out: List<V> = [v]
    while par[out[out.len - 1]] != None: out.push(par[out[out.len - 1]]!)
    out
fn path(par: [V]?V, a: V, b: V) -> ?List<V>:
    let ca = chain(par, a)
    let cb = chain(par, b)
    if ca[ca.len - 1] != cb[cb.len - 1]: return None
    var i = ca.len - 1
    var k = cb.len - 1
    while i > 0 and k > 0 and ca[i - 1] == cb[k - 1]:
        i -= 1
        k -= 1
    var out: List<V> = []
    for t in 0..=i: out.push(ca[t])
    var r = k - 1
    while r >= 0:
        out.push(cb[r])
        r -= 1
    Some(out)
var ok = true
for a in V:
    let ca = chain(par, a)
    for limit in 0..40:
        var want: ?V = None
        for u in ca: if u.to_idx >= limit: want = Some(u)
        if j.jump_while(a, u => u.to_idx >= limit) != want: ok = false
    for b in V:
        let p = path(par, a, b)
        if p == None:
            if j.dist(a, b) != None or j.jump(a, b, 0) != None: ok = false
        else:
            let q = p!
            if j.dist(a, b) != Some(q.len - 1): ok = false
            for k in 0..=q.len: if j.jump(a, b, k) != (if k < q.len: Some(q[k]) else: None): ok = false
print(yes(ok))
