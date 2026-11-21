// AtCoder WTF 2026 D: find the lexicographically largest reachable permutation
// <= Q by extending the prefix against a monotone stack of (max_p, max_v)
// constraints, then greedily packing remaining values into suffix blocks.
struct Cond(max_p: Int, max_v: Int)

@!multitest
let n: Int = read()
let p = [0..n] _ => read::<Int>() - 1
var q = [0..n] _ => read::<Int>() - 1

var inv_p: [0..n]Int = 0
for i in 0..n: inv_p[p[i]] = i

var used: [0..n]Bool = false
var st = 0
while st < n and p[st] == q[st]:
    used[q[st]] = true
    st += 1

var pref_mins: [0..=n]Int = 0
for v in 0..n:
    pref_mins[v] = if used[v]: n else: inv_p[v]
    if v > 0: pref_mins[v] min= pref_mins[v - 1]
pref_mins[n] = -1

var conds: List<Cond> = [Cond(-1, n)]
var found_move = false

fn try_place(i: Int) -> Bool:
    let v = q[i]
    let pos = inv_p[v]
    if i == st and v == p[i]:
        used[v] = true
        return true
    let n_found = found_move or (i - 1 >= st and v < q[i - 1] and pos < inv_p[q[i - 1]])
    if not n_found:
        var cap = min(q[i], q[i + 1])
        if pos < conds.last!.max_p: cap min= conds.last!.max_v
        if pos <= pref_mins[cap]: return false
        while pos < conds.last!.max_p: conds.pop()
        if cap < conds.last!.max_v: conds.push(Cond(pos, cap))
    elif v > conds.last!.max_v and pos < conds.last!.max_p:
        let cap = conds.last!.max_v
        if pos <= pref_mins[cap]: return false
        while pos < conds.last!.max_p: conds.pop()
        conds.push(Cond(pos, cap))
        found_move = n_found
    else:
        while pos < conds.last!.max_p: conds.pop()
        found_move = n_found
    used[v] = true
    true

var i = st
while i < n:
    var broke = false
    while not try_place(i):
        broke = true
        q[i + 1] = n
        q[i] -= 1
        while used[q[i]]: q[i] -= 1
    i += 1
    if broke: break

var block_starts: List<Int> = [i]
var p_to_block: [0..n]?Int = none
var last_p_thresh = n
while conds.last!.max_p != -1:
    var cond = conds.pop()!
    let p_thresh = pref_mins[cond.max_v]
    while p_thresh == pref_mins[conds.last!.max_v]:
        cond.max_p = conds.last!.max_p
        conds.pop()
    cond.max_p min= last_p_thresh

    var min_p = p_thresh
    var min_p_val = p[min_p]
    for pos in min_p + 1..cond.max_p if not used[p[pos]]:
        let v = p[pos]
        if v > cond.max_v: break
        if v >= min_p_val:
            min_p = pos
            min_p_val = v

    while min_p < conds.last!.max_p: conds.pop()
    let block_idx = block_starts.len - 1
    var en = block_starts.last!
    for pos in (min_p..last_p_thresh).rev if not used[p[pos]]:
        if p[pos] >= min_p_val:
            p_to_block[pos] = block_idx
            en += 1
    block_starts.push(en)
    last_p_thresh = p_thresh

for v in (0..n).rev if not used[v]:
    let b = p_to_block[inv_p[v]] ?? (block_starts.len - 1)
    q[block_starts[b]] = v
    block_starts[b] += 1

print([x + 1 for x in q])
