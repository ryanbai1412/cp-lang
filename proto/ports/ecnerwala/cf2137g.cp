@!multitest
let (n, m, q): (Int, Int, Int) = read()
var out_deg: [0..n]Int = 0
var in_e: [0..n]List<Int> = []
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    out_deg[u - 1] += 1
    in_e[v - 1].push(u - 1)
let orig_out = out_deg

var val: [0..n]Int = 0
var freq: [0..n, 0..4]Int = 0
var is_red: [0..n]Bool = false

fn get_val(v: Int) -> Int:
    if is_red[v]: return 3
    if orig_out[v] == 0: return 0
    if freq[v, 2] > 0 or (freq[v, 0] > 0 and freq[v, 3] > 0): return 1
    if freq[v, 3] > 0: return 3
    if freq[v, 0] > 0: return 0
    2

var order: Deque<Int> = Deque()
for v in 0..n:
    if out_deg[v] == 0: order.push(v)
while let v = order.pop_front():
    val[v] = get_val(v)
    for u in in_e[v]:
        freq[u, val[v]] += 1
        out_deg[u] -= 1
        if out_deg[u] == 0: order.push(u)

fn propagate(v: Int):
    let old_v = val[v]
    let new_v = get_val(v)
    if new_v == old_v: return
    val[v] = new_v
    for u in in_e[v]:
        freq[u, old_v] -= 1
        freq[u, new_v] += 1
        propagate(u)

for _ in 0..q:
    let (op, v0): (Int, Int) = read()
    let v = v0 - 1
    if op == 1:
        is_red[v] = true
        propagate(v)
    else:
        print(if val[v] <= 1: "YES" else: "NO")
