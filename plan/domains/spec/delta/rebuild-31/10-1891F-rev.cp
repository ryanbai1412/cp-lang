// CF 1891F A Growing Tree, revised: type 3 v prints v's value now.
// Build the final tree offline. A subtree add is a range add over preorder
// positions; a vertex created later must not see earlier adds, so its value
// is the final read minus the read at its creation.
@!multitest
let q = read<1..=5e5>()
var ops = List<(t: Int, v: Int, x: Int)>()
for _ in 0..q:
    let t = read<1..=3>()
    if t == 1: ops.push((t: 1, v: read<Int>(), x: 0))
    elif t == 2: ops.push((t: 2, v: read<Int>(), x: read<-1e9..=1e9>()))
    else: ops.push((t: 3, v: read<Int>(), x: 0))
let sz = 1 + ops.count(o => o.t == 1)
domain V = 1..=sz
var par: [V]?V = None
var made = 1
for o in ops:
    if o.t == 1:
        made += 1
        par[V(made)] = V(o.v)
let tree = Rooted.from_parents(par)
var add = DualFenwick(tree.reorder([V] _ => 0))
var base: [V]Int = 0
made = 1
for o in ops:
    if o.t == 1:
        made += 1
        base[V(made)] = add.get(tree.pos(V(made)))
    elif o.t == 2: add.add(tree.sub(V(o.v)), o.x)
    else: print(add.get(tree.pos(V(o.v))) - base[V(o.v)])
print([add.get(tree.pos(v)) - base[v] for v in V])
